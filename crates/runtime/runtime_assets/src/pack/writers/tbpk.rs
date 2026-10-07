use std::path::{Path, PathBuf};

use crate::pack::{validate_id, writers::ManifestSource};

#[derive(Default)]
pub struct PackWriter {
    manifest_source: Option<ManifestSource>,
}

struct AssetEntries {
    client: Vec<String>,
    server: Vec<String>,
    shared: Vec<String>,
}

impl PackWriter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn read_manifest(&mut self, path: PathBuf) -> Result<(), String> {
        self.manifest_source = Some(ManifestSource::from_path(&path)?);
        Ok(())
    }

    fn collect_asset_entries(&self) -> AssetEntries {
        let source = self.manifest_source.as_ref().expect("manifest source should be some before calling this, not intended to call before loading");

        let shared = source.get_assets().keys().cloned().collect::<Vec<_>>();
        let client = source
            .get_client()
            .get_assets()
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        let server = source
            .get_server()
            .get_assets()
            .keys()
            .cloned()
            .collect::<Vec<_>>();

        AssetEntries {
            client,
            server,
            shared,
        }
    }

    pub fn prep_assets(&mut self) -> Result<(), String> {
        if self.manifest_source.is_none() {
            return Err(
                "Manifest not loaded, load manifest before attempting to prep assets".into(),
            );
        }

        let asset_entries = self.collect_asset_entries();

        for id in asset_entries
            .shared
            .iter()
            .chain(&asset_entries.client)
            .chain(&asset_entries.server)
        {
            validate_id(id)?;
        }

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
    fn rejects_invalid_asset_id() {
        // basic1 has `self:cube.material`, and `.` isn't allowed in ids
        let mut writer = PackWriter::new();
        writer.read_manifest(example_manifest()).unwrap();

        let err = writer.prep_assets().unwrap_err();
        assert!(err.contains("self:cube.material"), "{err}");
    }
}
