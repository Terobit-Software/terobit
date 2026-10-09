use crate::pack::{Handle, writers::ManifestSource};
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};
use tracing::info;

#[derive(Default)]
pub struct PackWriter {
    manifest_source: Option<ManifestSource>,
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
    client: Vec<AssetInfo>,
    server: Vec<AssetInfo>,
    shared: Vec<AssetInfo>,
}

struct AssetInfo {
    handle: Handle,
    /// on load, if they do not match panic
    size: u64,
}

impl PackWriter {
    pub fn new() -> Self {
        Self::default()
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

    fn load_asset_group_info(&self, group: Vec<Handle>) -> Result<Vec<AssetInfo>, String> {
        let source = self.manifest_source.as_ref().expect("manifest source should be some before calling this, not intended to call before loading");

        group
            .into_iter()
            .map(|handle| {
                let path = source.resolve_handle_to_path(&handle)?;
                let size = std::fs::metadata(&path)
                    .map_err(|e| {
                        format!("failed to read `{}` for `{handle}`: {e}", path.display())
                    })?
                    .len();

                info!(
                    "loaded asset file info at path {} and with size {}",
                    path.to_string_lossy(),
                    size
                );

                Ok(AssetInfo { handle, size })
            })
            .collect()
    }

    fn load_asset_info(&mut self, handles: AssetHandles) -> Result<AssetInfoHolder, String> {
        Ok(AssetInfoHolder {
            client: self.load_asset_group_info(handles.client)?,
            server: self.load_asset_group_info(handles.server)?,
            shared: self.load_asset_group_info(handles.shared)?,
        })
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

    #[test]
    fn prep_assets_without_manifest_is_error() {
        assert!(PackWriter::new().prep_assets().is_err());
    }

    #[test]
    fn rejects_invalid_asset_handle() {
        // basic1 has `self:cube.material`, and `.` isn't allowed in handles
        let mut writer = PackWriter::new();
        writer.read_manifest(example_manifest()).unwrap();

        let err = writer.prep_assets().unwrap_err();
        assert!(err.contains("self:cube.material"), "{err}");
    }
}
