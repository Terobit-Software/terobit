use std::path::{Path, PathBuf};

use crate::writers::ManifestSource;

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

    fn collect_asset_entries() -> AssetEntries {}

    pub fn prep_assets(&mut self) -> Result<(), String> {
        if self.manifest_source.is_none() {
            return Err(
                "Manifest not loaded, load manifest before attempting to prep assets".into(),
            );
        }

        Ok(())
    }
}
