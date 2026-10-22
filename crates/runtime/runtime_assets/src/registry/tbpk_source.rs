use std::path::Path;
use std::{collections::HashMap, path::PathBuf};

use crate::pack::{AssetHandle, IndexV1, ManifestV1};
use crate::registry::RootLocation;

pub struct TbpkSource {
    manifest: ManifestV1,
    root: RootLocation,
    handle_to_path: HashMap<AssetHandle, Box<Path>>,
}

impl TbpkSource {
    pub fn new(manifest: ManifestV1, root: RootLocation) -> Self {
        let mut handle_to_path = HashMap::new();

        for scr in &manifest.scripts {
            handle_to_path.insert(
                scr.id.parse().unwrap(),
                PathBuf::from(manifest.files[scr.file as usize].path.clone()).into_boxed_path(),
            );
        }

        for ass in &manifest.assets {
            handle_to_path.insert(
                ass.id.parse().unwrap(),
                PathBuf::from(manifest.files[ass.file as usize].path.clone()).into_boxed_path(),
            );
        }

        Self {
            manifest,
            root,
            handle_to_path,
        }
    }

    pub fn get_manifest(&self) -> &ManifestV1 {
        &self.manifest
    }

    pub fn get_root(&self) -> &RootLocation {
        &self.root
    }

    pub fn resolve_handle_to_path(&self, handle: &AssetHandle) -> Option<&Box<Path>> {
        self.handle_to_path.get(handle)
    }
}
