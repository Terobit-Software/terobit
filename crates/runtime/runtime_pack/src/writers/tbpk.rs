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

        Ok(())
    }
}
