use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::ManifestSource;
use crate::pack::{AssetHandle, Target};

/// one side of a loaded pack: the assets and scripts it declares and the files it can read
#[derive(Default)]
pub struct TbpkSource {
    /// where the pack's files are on disk
    root: PathBuf,
    /// asset id to pack path
    assets: HashMap<String, String>,
    /// script id to pack path
    scripts: HashMap<String, String>,
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
            scripts: manifest
                .scripts_for(target)
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

    /// every script this side runs, sorted so they load in the same order every run
    pub fn script_handles(&self) -> Vec<AssetHandle> {
        let mut handles: Vec<AssetHandle> = self
            .scripts
            .keys()
            // the manifest checked every id when it was read
            .filter_map(|id| match id.parse() {
                Ok(handle) => Some(handle),
                Err(e) => {
                    tracing::error!(
                        "Script id `{id}` isn't a valid handle, this is a bug in the manifest validation: {}",
                        snafu::Report::from_error(&e)
                    );
                    None
                }
            })
            .collect();
        handles.sort();
        handles
    }

    /// the pack path of a script
    pub fn resolve_script(&self, handle: &AssetHandle) -> Option<&str> {
        self.scripts.get(&handle.to_string()).map(String::as_str)
    }

    /// where a pack path is on disk, `None` if it isn't one of this side's files
    pub fn full_path(&self, path: &Path) -> Option<PathBuf> {
        self.files.contains(path).then(|| self.root.join(path))
    }
}
