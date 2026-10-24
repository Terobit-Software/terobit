use serde::Deserialize;
use snafu::{OptionExt, ResultExt, Snafu, ensure};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use tracing::{info, warn};

use crate::pack::{
    Access, AssetEntry, AssetHandle, FileEntry, HandleError, IndexRef, ManifestV1, PathError,
    ScriptEntry, Target, validate_path,
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManifestSource {
    realm_id: u64,
    owner_id: u64,
    pack_version: u32,
    access: Access,
    #[serde(default)]
    assets: BTreeMap<String, String>,
    #[serde(default)]
    client: SideAssets,
    #[serde(default)]
    server: SideAssets,
    /// directory the manifest was read from, asset paths are relative to it
    #[serde(skip)]
    root: PathBuf,
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub(crate) struct SideAssets {
    #[serde(default)]
    // btree instead of hashmap for determinism in builds
    assets: BTreeMap<String, String>,
    #[serde(default)]
    scripts: BTreeMap<String, String>,
}

/// a manifest.toml that couldn't be loaded
#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum ManifestLoadError {
    #[snafu(display("failed to read manifest file `{}`", path.display()))]
    ReadFile {
        path: PathBuf,
        source: std::io::Error,
    },

    #[snafu(display(
        "manifest `{}` is not a valid manifest, check its toml syntax, field names and value types",
        path.display()
    ))]
    Parse {
        path: PathBuf,
        source: toml::de::Error,
    },

    #[snafu(display("manifest `{}` was parsed but failed validation", path.display()))]
    Invalid {
        path: PathBuf,
        #[snafu(source(from(ManifestValidateError, Box::new)))]
        source: Box<ManifestValidateError>,
    },
}

/// a parsed manifest that breaks the manifest or conventions rules
#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum ManifestValidateError {
    #[snafu(display("`realm_id` is 0, but realm ids start at 1"))]
    ZeroRealmId,

    #[snafu(display("`owner_id` is 0, but owner ids start at 1"))]
    ZeroOwnerId,

    #[snafu(display("asset id `{id}` in [{table}] is not a valid handle"))]
    InvalidHandle {
        table: &'static str,
        id: String,
        #[snafu(source(from(HandleError, Box::new)))]
        source: Box<HandleError>,
    },

    #[snafu(display("`{id}` in [{table}] points to `{path}`, which is not a valid pack path"))]
    InvalidPath {
        table: &'static str,
        id: String,
        path: String,
        #[snafu(source(from(PathError, Box::new)))]
        source: Box<PathError>,
    },

    #[snafu(display(
        "`{id}` is declared in both [assets] and [{}.assets], a handle can only be declared once per side",
        side_table(*target)
    ))]
    SharedAndSide { id: String, target: Target },

    #[snafu(display(
        "{target:?} script `{id}` points to `{path}`, but scripts must be `.wasm` files"
    ))]
    ScriptNotWasm {
        id: String,
        path: String,
        target: Target,
    },
}

/// one side's manifest or file list couldn't be built from the source manifest
#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum ManifestSideError {
    #[snafu(display(
        "failed to read the size of {target:?} file `{pack_path}` at `{}`, check that it exists and is readable",
        full_path.display()
    ))]
    FileMetadata {
        target: Target,
        pack_path: String,
        full_path: PathBuf,
        source: std::io::Error,
    },

    #[snafu(display(
        "{target:?} file `{path}` is referenced by an asset or script but missing from the sorted file list, this is a bug in the pack writer"
    ))]
    MissingFromFileList { target: Target, path: String },

    #[snafu(display(
        "{target:?} side has {count} files, but a pack can index at most {} files",
        u32::MAX
    ))]
    TooManyFiles { target: Target, count: usize },
}

/// the manifest table a side's assets and scripts live under
fn side_table(target: Target) -> &'static str {
    match target {
        Target::Client => "client",
        Target::Server => "server",
    }
}

impl ManifestSource {
    #[tracing::instrument(skip_all, fields(manifest = %path.display()))]
    pub fn from_path(path: &std::path::Path) -> Result<Self, ManifestLoadError> {
        let text = std::fs::read_to_string(path).context(ReadFileSnafu { path })?;

        match path.canonicalize() {
            Ok(canon) => info!("Read manifest at path: {}", canon.display()),
            Err(e) => warn!(
                "Read manifest at path {}, but couldn't canonicalize it ({e}), asset paths will be resolved relative to it as given",
                path.display()
            ),
        }

        let mut manifest = toml::from_str::<Self>(&text).context(ParseSnafu { path })?;

        info!("Converted str source to manifest successfully");

        manifest.root = path.parent().map(Into::into).unwrap_or_default();
        manifest.validate().context(InvalidSnafu { path })?;

        info!(
            "Manifest is realm {}, owner {}, pack version {}, access {:?}",
            manifest.realm_id, manifest.owner_id, manifest.pack_version, manifest.access
        );
        info!(
            "Manifest has {} shared assets, {} client assets, {} client scripts, {} server assets, {} server scripts",
            manifest.assets.len(),
            manifest.client.assets.len(),
            manifest.client.scripts.len(),
            manifest.server.assets.len(),
            manifest.server.scripts.len()
        );
        Ok(manifest)
    }

    /// ids are never 0, every handle and path follows the conventions doc, and each side
    /// follows the source.md rules
    fn validate(&self) -> Result<(), ManifestValidateError> {
        ensure!(self.realm_id != 0, ZeroRealmIdSnafu);
        ensure!(self.owner_id != 0, ZeroOwnerIdSnafu);

        for (table, entries) in [
            ("assets", &self.assets),
            ("client.assets", &self.client.assets),
            ("client.scripts", &self.client.scripts),
            ("server.assets", &self.server.assets),
            ("server.scripts", &self.server.scripts),
        ] {
            for (id, path) in entries {
                id.parse::<AssetHandle>()
                    .context(InvalidHandleSnafu { table, id })?;
                validate_path(path).context(InvalidPathSnafu { table, id, path })?;
            }
        }

        for target in [Target::Client, Target::Server] {
            let side = self.side(target);
            for id in side.assets.keys() {
                ensure!(
                    !self.assets.contains_key(id),
                    SharedAndSideSnafu { id, target }
                );
            }
            for (id, path) in &side.scripts {
                ensure!(
                    path.ends_with(".wasm"),
                    ScriptNotWasmSnafu { id, path, target }
                );
            }
        }

        Ok(())
    }

    pub fn get_realm_id(&self) -> &u64 {
        &self.realm_id
    }

    pub fn get_owner_id(&self) -> &u64 {
        &self.owner_id
    }

    pub fn get_pack_version(&self) -> &u32 {
        &self.pack_version
    }

    pub(crate) fn get_access(&self) -> &Access {
        &self.access
    }

    pub fn get_assets(&self) -> &BTreeMap<String, String> {
        &self.assets
    }

    pub(crate) fn get_client(&self) -> &SideAssets {
        &self.client
    }

    pub(crate) fn get_server(&self) -> &SideAssets {
        &self.server
    }

    /// directory the manifest was read from, pack paths are relative to it
    pub fn get_root(&self) -> &Path {
        &self.root
    }

    fn side(&self, target: Target) -> &SideAssets {
        match target {
            Target::Client => &self.client,
            Target::Server => &self.server,
        }
    }

    /// `(id, path)` of every asset one side can see: the shared ones, then that side's.
    /// the client and server namespaces are separate
    pub(crate) fn assets_for(&self, target: Target) -> impl Iterator<Item = (&String, &String)> {
        self.assets.iter().chain(&self.side(target).assets)
    }

    /// every path one side references, deduplicated and sorted in byte order
    pub(crate) fn paths_for(&self, target: Target) -> BTreeSet<&str> {
        self.assets_for(target)
            .chain(&self.side(target).scripts)
            .map(|(_, path)| path.as_str())
            .collect()
    }

    /// one side's files: the shared assets plus that side's assets and scripts, sorted by
    /// path in byte order, with sizes read from disk
    pub(crate) fn get_side_files(
        &self,
        target: Target,
    ) -> Result<Vec<FileEntry>, ManifestSideError> {
        let files = self
            .paths_for(target)
            .into_iter()
            .map(|path| {
                let full_path = self.root.join(path);
                let size = std::fs::metadata(&full_path)
                    .context(FileMetadataSnafu {
                        target,
                        pack_path: path,
                        full_path: &full_path,
                    })?
                    .len();
                Ok(FileEntry {
                    path: path.to_owned(),
                    size,
                })
            })
            .collect::<Result<Vec<_>, ManifestSideError>>()?;

        for file in &files {
            info!("{target:?} file `{}` is {} bytes", file.path, file.size);
        }
        info!("{target:?} side has {} files", files.len());
        Ok(files)
    }

    /// one side's manifest. `files` must be that side's `get_side_files`, the list the
    /// index was chunked from
    pub(crate) fn get_v1_side(
        &self,
        target: Target,
        files: Vec<FileEntry>,
        index: IndexRef,
    ) -> Result<ManifestV1, ManifestSideError> {
        ensure!(
            u32::try_from(files.len()).is_ok(),
            TooManyFilesSnafu {
                target,
                count: files.len()
            }
        );

        let file_index = |path: &str| -> Result<u32, ManifestSideError> {
            let i = files
                .binary_search_by(|f| f.path.as_str().cmp(path))
                .ok()
                .context(MissingFromFileListSnafu { target, path })?;
            // fits, the file count was checked above
            Ok(i as u32)
        };

        let mut assets = self
            .assets_for(target)
            .map(|(id, path)| {
                Ok(AssetEntry {
                    id: id.clone(),
                    file: file_index(path)?,
                })
            })
            .collect::<Result<Vec<_>, ManifestSideError>>()?;
        assets.sort_by(|a, b| a.id.cmp(&b.id));

        let scripts = self
            .side(target)
            .scripts
            .iter()
            .map(|(id, path)| {
                Ok(ScriptEntry {
                    id: id.clone(),
                    file: file_index(path)?,
                })
            })
            .collect::<Result<Vec<_>, ManifestSideError>>()?;

        Ok(ManifestV1 {
            target,
            realm_id: self.realm_id,
            owner_id: self.owner_id,
            access: self.access,
            pack_version: self.pack_version,
            files,
            assets,
            scripts,
            index,
        })
    }
}

impl SideAssets {
    pub fn get_assets(&self) -> &BTreeMap<String, String> {
        &self.assets
    }

    pub fn get_scripts(&self) -> &BTreeMap<String, String> {
        &self.scripts
    }
}

#[cfg(test)]
mod struct_tests {
    use super::*;
    use std::path::{Path, PathBuf};

    fn example_manifest() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../tests/examples/source_to_tbpk/basic1/manifest.toml")
    }

    fn parse(name: &str, text: &str) -> Result<ManifestSource, ManifestLoadError> {
        let path = std::env::temp_dir().join(format!(
            "terobit_manifest_{}_{}.toml",
            std::process::id(),
            name
        ));
        std::fs::write(&path, text).unwrap();
        let result = ManifestSource::from_path(&path);
        std::fs::remove_file(&path).unwrap();
        result
    }

    #[test]
    fn reads_example_manifest() {
        let manifest = ManifestSource::from_path(&example_manifest()).unwrap();

        assert_eq!(manifest.realm_id, 1);
        assert_eq!(manifest.owner_id, 1);
        assert_eq!(manifest.pack_version, 1);
        assert!(matches!(manifest.access, Access::Free));
        assert_eq!(
            manifest.assets.get("self:cube").map(String::as_str),
            Some("meshes/mesh.obj")
        );
        assert_eq!(
            manifest
                .assets
                .get("self:cube/material")
                .map(String::as_str),
            Some("meshes/mesh.mtl")
        );
        assert_eq!(manifest.assets.len(), 2);
        assert_eq!(
            manifest.client.assets.get("self:car").map(String::as_str),
            Some("images/car.gif")
        );
        assert_eq!(manifest.client.assets.len(), 1);
        assert_eq!(
            manifest.server.assets.get("self:music").map(String::as_str),
            Some("audio/music.mp3")
        );
        assert_eq!(manifest.server.assets.len(), 1);
    }

    #[test]
    fn rejects_invalid_handle_in_example_manifest() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../tests/examples/source_to_tbpk/basic1_should_fail/manifest.toml");

        let err = ManifestSource::from_path(&path).err().unwrap();
        assert!(
            matches!(&err, ManifestLoadError::Invalid { source, .. }
                if matches!(**source, ManifestValidateError::InvalidHandle { .. })),
            "{err:?}"
        );
        let report = snafu::Report::from_error(&err).to_string();
        assert!(report.contains("self:cube.material"), "{report}");
    }

    #[test]
    fn asset_tables_default_to_empty() {
        let manifest = parse(
            "minimal",
            "realm_id = 2\nowner_id = 3\npack_version = 4\naccess = \"licensed\"\n",
        )
        .unwrap();

        assert_eq!(manifest.realm_id, 2);
        assert_eq!(manifest.owner_id, 3);
        assert_eq!(manifest.pack_version, 4);
        assert!(matches!(manifest.access, Access::Licensed));
        assert!(manifest.assets.is_empty());
        assert!(manifest.client.assets.is_empty());
        assert!(manifest.server.assets.is_empty());
    }

    #[test]
    fn empty_side_table_is_accepted() {
        let manifest = parse(
            "empty_side",
            "realm_id = 1\nowner_id = 1\npack_version = 1\naccess = \"free\"\n[client]\n",
        )
        .unwrap();

        assert!(manifest.client.assets.is_empty());
    }

    #[test]
    fn rejects_unknown_top_level_field() {
        assert!(
            parse(
                "unknown_field",
                "realm_id = 1\nowner_id = 1\npack_version = 1\naccess = \"free\"\nextra = 1\n",
            )
            .is_err()
        );
    }

    #[test]
    fn rejects_unknown_side_field() {
        assert!(
            parse(
                "unknown_side_field",
                "realm_id = 1\nowner_id = 1\npack_version = 1\naccess = \"free\"\n[client]\nextra = 1\n",
            )
            .is_err()
        );
    }

    #[test]
    fn rejects_unknown_access_level() {
        assert!(
            parse(
                "bad_access",
                "realm_id = 1\nowner_id = 1\npack_version = 1\naccess = \"paid\"\n",
            )
            .is_err()
        );
    }

    #[test]
    fn access_level_is_lowercase_only() {
        assert!(
            parse(
                "uppercase_access",
                "realm_id = 1\nowner_id = 1\npack_version = 1\naccess = \"Free\"\n",
            )
            .is_err()
        );
    }

    #[test]
    fn rejects_missing_required_field() {
        assert!(
            parse(
                "missing_field",
                "realm_id = 1\nowner_id = 1\naccess = \"free\"\n"
            )
            .is_err()
        );
    }

    #[test]
    fn rejects_negative_id() {
        assert!(
            parse(
                "negative_id",
                "realm_id = -1\nowner_id = 1\npack_version = 1\naccess = \"free\"\n",
            )
            .is_err()
        );
    }

    #[test]
    fn reads_scripts() {
        let manifest = parse(
            "scripts",
            "realm_id = 1\nowner_id = 1\npack_version = 1\naccess = \"free\"\n\
             [client.scripts]\n\"self:main\" = \"scripts/client.wasm\"\n\
             [server.scripts]\n\"self:main\" = \"scripts/server.wasm\"\n",
        )
        .unwrap();

        assert_eq!(
            manifest.client.scripts.get("self:main").map(String::as_str),
            Some("scripts/client.wasm")
        );
        assert_eq!(
            manifest.server.scripts.get("self:main").map(String::as_str),
            Some("scripts/server.wasm")
        );
    }

    #[test]
    fn rejects_zero_id() {
        assert!(
            parse(
                "zero_realm_id",
                "realm_id = 0\nowner_id = 1\npack_version = 1\naccess = \"free\"\n",
            )
            .is_err()
        );
        assert!(
            parse(
                "zero_owner_id",
                "realm_id = 1\nowner_id = 0\npack_version = 1\naccess = \"free\"\n",
            )
            .is_err()
        );
    }

    #[test]
    fn rejects_pack_version_over_u32() {
        assert!(
            parse(
                "big_pack_version",
                "realm_id = 1\nowner_id = 1\npack_version = 4294967296\naccess = \"free\"\n",
            )
            .is_err()
        );
    }

    #[test]
    fn rejects_invalid_path() {
        for (name, table) in [
            ("bad_shared_path", "[assets]"),
            ("bad_client_path", "[client.assets]"),
            ("bad_server_script_path", "[server.scripts]"),
        ] {
            let err = parse(
                name,
                &format!(
                    "realm_id = 1\nowner_id = 1\npack_version = 1\naccess = \"free\"\n\
                     {table}\n\"self:x\" = \"../outside.png\"\n"
                ),
            )
            .err()
            .expect("path should be rejected");
            let report = snafu::Report::from_error(&err).to_string();
            assert!(report.contains("../outside.png"), "{report}");
        }
    }

    #[test]
    fn rejects_invalid_toml() {
        assert!(matches!(
            parse("invalid_toml", "realm_id = \n"),
            Err(ManifestLoadError::Parse { .. })
        ));
    }

    #[test]
    fn missing_file_is_error() {
        let path = std::env::temp_dir().join("terobit_manifest_does_not_exist.toml");
        assert!(matches!(
            ManifestSource::from_path(&path),
            Err(ManifestLoadError::ReadFile { .. })
        ));
    }

    fn validate_error(name: &str, tables: &str) -> ManifestValidateError {
        let err = parse(
            name,
            &format!("realm_id = 1\nowner_id = 1\npack_version = 1\naccess = \"free\"\n{tables}"),
        )
        .err()
        .expect("manifest should be rejected");
        match err {
            ManifestLoadError::Invalid { source, .. } => *source,
            other => panic!("expected a validation error, got {other:?}"),
        }
    }

    #[test]
    fn rejects_asset_declared_shared_and_on_a_side() {
        let err = validate_error(
            "shared_and_side",
            "[assets]\n\"self:x\" = \"a.png\"\n[client.assets]\n\"self:x\" = \"b.png\"\n",
        );
        assert!(
            matches!(
                err,
                ManifestValidateError::SharedAndSide {
                    target: Target::Client,
                    ..
                }
            ),
            "{err:?}"
        );
    }

    #[test]
    fn rejects_script_that_isnt_wasm() {
        let err = validate_error(
            "script_not_wasm",
            "[server.scripts]\n\"self:main\" = \"scripts/main.lua\"\n",
        );
        assert!(
            matches!(
                err,
                ManifestValidateError::ScriptNotWasm {
                    target: Target::Server,
                    ..
                }
            ),
            "{err:?}"
        );
    }
}
