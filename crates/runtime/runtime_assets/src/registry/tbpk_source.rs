use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::ManifestSource;
use crate::pack::{AssetHandle, Target};

/// one side of a loaded pack: the assets it declares and the files it can read
#[derive(Default)]
pub struct TbpkSource {
    /// where the pack's files are on disk
    root: PathBuf,
    /// asset id to pack path
    assets: HashMap<String, String>,
    files: HashSet<PathBuf>,
}

impl TbpkSource {
    /// a dev directory read in place. like a built pack, it only has the shared entries and
    /// those of `target`
    pub fn dev(manifest: &ManifestSource, target: Target) -> Self {
        Self {
            root: manifest.get_root().to_owned(),
            assets: manifest
                .assets_for(target)
                .map(|(id, path)| (id.clone(), path.clone()))
                .collect(),
            files: manifest
                .paths_for(target)
                .into_iter()
                .map(PathBuf::from)
                .collect(),
        }
    }

    /// the pack path of an asset
    pub fn resolve(&self, handle: &AssetHandle) -> Option<&str> {
        self.assets.get(&handle.to_string()).map(String::as_str)
    }

    /// where a pack path is on disk, `None` if it isn't one of this side's files
    pub fn full_path(&self, path: &Path) -> Option<PathBuf> {
        self.files.contains(path).then(|| self.root.join(path))
    }
}
