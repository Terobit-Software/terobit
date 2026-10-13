use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::pack::{Access, Handle, validate_path};

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

impl ManifestSource {
    pub fn from_path(path: &std::path::Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;

        if let Ok(canon) = path.canonicalize() {
            tracing::info!("Read manifest at path: {}", canon.to_string_lossy());
        } else {
            tracing::error!("Failed to make manifest read path canon")
        }

        let mut manifest = toml::from_str::<Self>(&text).map_err(|e| e.to_string())?;

        tracing::info!("Converted str source to manifest successfully");

        manifest.root = path.parent().map(Into::into).unwrap_or_default();
        manifest.validate()?;
        Ok(manifest)
    }

    /// ids are never 0, and every path follows the path rules in the conventions doc
    fn validate(&self) -> Result<(), String> {
        if self.realm_id == 0 {
            return Err("realm_id can't be 0".into());
        }
        if self.owner_id == 0 {
            return Err("owner_id can't be 0".into());
        }

        for table in [
            &self.assets,
            &self.client.assets,
            &self.client.scripts,
            &self.server.assets,
            &self.server.scripts,
        ] {
            for (id, path) in table {
                validate_path(path).map_err(|e| format!("`{id}`: {e}"))?;
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

    /// the pack path of an asset on one side. the client and server namespaces are
    /// separate, so only shared assets and that side's assets are searched
    pub(crate) fn resolve_asset_path<'a>(
        &'a self,
        side: &'a SideAssets,
        handle: &Handle,
    ) -> Result<&'a str, String> {
        let key = handle.to_string();

        [&self.assets, &side.assets]
            .into_iter()
            .find_map(|assets| assets.get(&key))
            .map(String::as_str)
            .ok_or_else(|| format!("handle with name '{handle}' is not found in the manifest"))
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

    fn parse(name: &str, text: &str) -> Result<ManifestSource, String> {
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
            assert!(err.contains("../outside.png"), "{err}");
        }
    }

    #[test]
    fn rejects_invalid_toml() {
        assert!(parse("invalid_toml", "realm_id = \n").is_err());
    }

    #[test]
    fn missing_file_is_error() {
        let path = std::env::temp_dir().join("terobit_manifest_does_not_exist.toml");
        assert!(ManifestSource::from_path(&path).is_err());
    }
}
