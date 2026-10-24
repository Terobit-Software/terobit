use bevy::ecs::resource::Resource;
use std::path::{Path, PathBuf};
use std::sync::{Arc, PoisonError, RwLock, RwLockReadGuard};

use crate::pack::AssetHandle;
use crate::registry::TbpkSource;

/// the loaded pack, shared by `PackAssets` and the `pack://` asset source
#[derive(Resource, Clone, Default)]
pub struct PackRegistry(Arc<RwLock<TbpkSource>>);

// the source is only ever replaced whole, so a panic can't leave it half written and a
// poisoned lock is safe to use
impl PackRegistry {
    /// replaces the loaded pack
    pub fn set(&self, source: TbpkSource) {
        *self.0.write().unwrap_or_else(PoisonError::into_inner) = source;
    }

    /// the pack path of an asset
    pub fn resolve(&self, handle: &AssetHandle) -> Option<String> {
        self.source().resolve(handle).map(str::to_owned)
    }

    /// where a pack path is on disk, `None` if the loaded pack doesn't have that file
    pub fn full_path(&self, path: &Path) -> Option<PathBuf> {
        self.source().full_path(path)
    }

    fn source(&self) -> RwLockReadGuard<'_, TbpkSource> {
        self.0.read().unwrap_or_else(PoisonError::into_inner)
    }
}
