use crate::pack::Handle;
use crate::pack::writers::{ManifestSource, SideAssets};
use crate::pack::{DEFAULT_CHUNK_SIZE, MAX_CHUNK_SIZE, MIN_CHUNK_SIZE};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::{Path, PathBuf};
use tracing::info;

#[derive(Default)]
pub struct PackWriter {
    manifest_source: Option<ManifestSource>,
    chunk_map: Option<FileChunkMap>,
    chunk_size: u32,
}

struct AssetHandles {
    client: Vec<Handle>,
    server: Vec<Handle>,
    shared: Vec<Handle>,
    validated: bool,
}

impl AssetHandles {
    pub fn validate(&mut self) -> Result<(), String> {
        let set = self.shared.clone().into_iter().collect::<HashSet<_>>();

        for v in &self.client {
            if set.contains(v) {
                return Err(format!("client list contains shared item: {}", v));
            }
        }

        for v in &self.server {
            if set.contains(v) {
                return Err(format!("server list contains shared item: {}", v));
            }
        }

        self.validated = true;

        info!("Asset handles validated");

        Ok(())
    }
}

struct AssetInfoHolder {
    client: SideInfo,
    server: SideInfo,
}

/// everything one side's pack holds
struct SideInfo {
    /// every path referenced by this side's entries, deduplicated and sorted by path
    files: Vec<FileInfo>,
    assets: Vec<AssetInfo>,
}

struct FileInfo {
    /// path inside the pack, see path rules in conventions
    path: String,
    /// on load, if they do not match panic
    size: u64,
}

struct AssetInfo {
    handle: Handle,
    /// index into the side's `files`
    file: u32,
}

struct ChunkItem {
    /// index into the side's `files`
    file: u32,
    /// in bytes
    offset: u32,
}

struct ChunkSection {
    items: Vec<ChunkItem>,
}

struct FileChunkMap {
    map: Vec<ChunkSection>,
}

impl PackWriter {
    pub fn new() -> Self {
        Self {
            chunk_size: DEFAULT_CHUNK_SIZE,
            ..Default::default()
        }
    }

    pub fn with_chunk_size(mut self, bytes: u32) -> Self {
        if bytes > MAX_CHUNK_SIZE || bytes < MIN_CHUNK_SIZE {
            panic!(
                "Chunk size is out of range, min: {MIN_CHUNK_SIZE}, max: {MAX_CHUNK_SIZE}, given: {bytes}"
            );
        }
        self.chunk_size = bytes;
        self
    }

    pub fn read_manifest(&mut self, path: PathBuf) -> Result<(), String> {
        self.manifest_source = Some(ManifestSource::from_path(&path)?);
        info!("Manifest read OK at path {}", path.to_string_lossy());
        Ok(())
    }

    fn collect_asset_handles(&self) -> Result<AssetHandles, String> {
        let source = self.manifest_source.as_ref().expect("manifest source should be some before calling this, not intended to call before loading");

        let parse = |assets: &BTreeMap<String, String>| {
            assets
                .keys()
                .map(|key| {
                    let v = key.parse::<Handle>();

                    if let Ok(val) = &v {
                        info!("parsed asset handle value {}", val);
                    }

                    v
                })
                .collect::<Result<Vec<_>, _>>()
        };

        Ok(AssetHandles {
            client: parse(source.get_client().get_assets())?,
            server: parse(source.get_server().get_assets())?,
            shared: parse(source.get_assets())?,
            validated: false,
        })
    }

    fn load_side_info(&self, side: &SideAssets, group: Vec<Handle>) -> Result<SideInfo, String> {
        let source = self.manifest_source.as_ref().expect("manifest source should be some before calling this, not intended to call before loading");

        let asset_paths = group
            .into_iter()
            .map(|handle| {
                let path = source.resolve_asset_path(side, &handle)?;
                Ok((handle, path))
            })
            .collect::<Result<Vec<_>, String>>()?;

        // btree so files are deduplicated and sorted by path in byte order, like the manifest
        let paths = asset_paths
            .iter()
            .map(|(_, path)| *path)
            .collect::<BTreeSet<_>>();

        let files = paths
            .into_iter()
            .map(|path| {
                let full_path = source.get_root().join(path);
                let metadata = std::fs::metadata(&full_path)
                    .map_err(|e| format!("failed to read `{}`: {e}", full_path.display()))?;
                if !metadata.is_file() {
                    return Err(format!("`{}` is not a file", full_path.display()));
                }

                info!(
                    "loaded file info at path {} and with size {}",
                    full_path.to_string_lossy(),
                    metadata.len()
                );

                Ok(FileInfo {
                    path: path.to_owned(),
                    size: metadata.len(),
                })
            })
            .collect::<Result<Vec<_>, String>>()?;

        let assets = asset_paths
            .into_iter()
            .map(|(handle, path)| {
                let file = files
                    .binary_search_by(|f| f.path.as_str().cmp(path))
                    .expect("every asset path is in files");
                AssetInfo {
                    handle,
                    file: file as u32,
                }
            })
            .collect();

        Ok(SideInfo { files, assets })
    }

    fn load_asset_info(&self, mut handles: AssetHandles) -> Result<AssetInfoHolder, String> {
        let source = self.manifest_source.as_ref().expect("manifest source should be some before calling this, not intended to call before loading");

        handles.server.extend_from_slice(&handles.shared);
        handles.client.extend_from_slice(&handles.shared);

        Ok(AssetInfoHolder {
            client: self.load_side_info(source.get_client(), handles.client)?,
            server: self.load_side_info(source.get_server(), handles.server)?,
        })
    }

    fn partition_chunks(&self, asset_info: AssetInfoHolder) -> FileChunkMap {
        let chunk_size = self.chunk_size;
    }

    pub fn prep_assets(&mut self) -> Result<(), String> {
        if self.manifest_source.is_none() {
            return Err(
                "Manifest not loaded, load manifest before attempting to prep assets".into(),
            );
        }

        let mut asset_handles = self.collect_asset_handles()?;
        info!("Finished loading asset handles");

        if let Err(msg) = asset_handles.validate() {
            return Err(format!("error while validating handles: {}", msg));
        }

        let asset_info = self.load_asset_info(asset_handles)?;
        info!("Finished loading asset info");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn example_manifest() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../tests/examples/source_to_tbpk/basic1/manifest.toml")
    }

    fn should_fail_example_manifest() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../tests/examples/source_to_tbpk/basic1_should_fail/manifest.toml")
    }

    #[test]
    fn prep_assets_without_manifest_is_error() {
        assert!(PackWriter::new().prep_assets().is_err());
    }

    /// writes `manifest.toml` and empty files at `paths` into a fresh temp dir
    fn temp_dev_dir(name: &str, manifest: &str, paths: &[&str]) -> PathBuf {
        let root =
            std::env::temp_dir().join(format!("terobit_writer_{}_{}", std::process::id(), name));
        let _ = std::fs::remove_dir_all(&root);
        for path in paths {
            let full = root.join(path);
            std::fs::create_dir_all(full.parent().unwrap()).unwrap();
            std::fs::write(full, path.as_bytes()).unwrap();
        }
        std::fs::write(root.join("manifest.toml"), manifest).unwrap();
        root
    }

    #[test]
    fn files_are_deduplicated_and_sorted_by_path() {
        let root = temp_dev_dir(
            "dedup",
            "realm_id = 1\nowner_id = 1\npack_version = 1\naccess = \"free\"\n\
             [assets]\n\"self:z\" = \"b/big.bin\"\n\"self:a\" = \"a.bin\"\n\
             [client.assets]\n\"self:again\" = \"a.bin\"\n",
            &["a.bin", "b/big.bin"],
        );

        let mut writer = PackWriter::new();
        writer.read_manifest(root.join("manifest.toml")).unwrap();
        let handles = writer.collect_asset_handles().unwrap();
        let info = writer.load_asset_info(handles).unwrap();

        let paths = |side: &SideInfo| {
            side.files
                .iter()
                .map(|f| f.path.clone())
                .collect::<Vec<_>>()
        };
        assert_eq!(paths(&info.client), ["a.bin", "b/big.bin"]);
        assert_eq!(paths(&info.server), ["a.bin", "b/big.bin"]);
        assert_eq!(info.client.assets.len(), 3);
        for asset in &info.client.assets {
            let expected = if asset.handle.id() == "z" { 1 } else { 0 };
            assert_eq!(asset.file, expected, "{}", asset.handle);
        }

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn client_and_server_ids_resolve_separately() {
        let root = temp_dev_dir(
            "sides",
            "realm_id = 1\nowner_id = 1\npack_version = 1\naccess = \"free\"\n\
             [client.assets]\n\"self:x\" = \"client.bin\"\n\
             [server.assets]\n\"self:x\" = \"server.bin\"\n",
            &["client.bin", "server.bin"],
        );

        let mut writer = PackWriter::new();
        writer.read_manifest(root.join("manifest.toml")).unwrap();
        let handles = writer.collect_asset_handles().unwrap();
        let info = writer.load_asset_info(handles).unwrap();

        assert_eq!(info.client.files[0].path, "client.bin");
        assert_eq!(info.server.files[0].path, "server.bin");

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_invalid_asset_handle() {
        // basic1 has `self:cube.material`, and `.` isn't allowed in handles
        let mut writer = PackWriter::new();
        writer
            .read_manifest(should_fail_example_manifest())
            .unwrap();

        let err = writer.prep_assets().unwrap_err();
        assert!(err.contains("self:cube.material"), "{err}");
    }
}
