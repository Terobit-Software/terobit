use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::pack::{Handle, writers::ManifestSource};

#[derive(Default)]
pub struct PackWriter {
    manifest_source: Option<ManifestSource>,
}

struct AssetEntries {
    client: Vec<Handle>,
    server: Vec<Handle>,
    shared: Vec<Handle>,
}

impl PackWriter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn read_manifest(&mut self, path: PathBuf) -> Result<(), String> {
        self.manifest_source = Some(ManifestSource::from_path(&path)?);
        Ok(())
    }

    fn collect_asset_entries(&self) -> Result<AssetEntries, String> {
        let source = self.manifest_source.as_ref().expect("manifest source should be some before calling this, not intended to call before loading");

        let parse = |assets: &BTreeMap<String, String>| {
            assets
                .keys()
                .map(|key| key.parse::<Handle>())
                .collect::<Result<Vec<_>, _>>()
        };

        Ok(AssetEntries {
            client: parse(source.get_client().get_assets())?,
            server: parse(source.get_server().get_assets())?,
            shared: parse(source.get_assets())?,
        })
    }

    pub fn prep_assets(&mut self) -> Result<(), String> {
        if self.manifest_source.is_none() {
            return Err(
                "Manifest not loaded, load manifest before attempting to prep assets".into(),
            );
        }

        let asset_entries = self.collect_asset_entries()?;

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
