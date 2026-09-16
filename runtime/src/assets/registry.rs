use std::collections::HashMap;
use std::path::Path;

use crate::assets::asset_error;
use crate::assets::raw_asset::BUILTIN_CUBE_PATH;
use crate::{MeshAsset, RawAsset};

pub(crate) type AssetId = usize;

pub(crate) struct AssetRegistry {
    cache: HashMap<Box<Path>, RawAsset>,
    meshes: HashMap<AssetId, MeshAsset>,
    mesh_ids: HashMap<Box<Path>, AssetId>,
    next_mesh_id: AssetId,
}

impl AssetRegistry {
    pub(crate) fn new() -> Result<Self, asset_error::AssetError> {
        let mut reg = Self {
            cache: HashMap::new(),
            meshes: HashMap::new(),
            mesh_ids: HashMap::new(),
            next_mesh_id: 0,
        };

        // loaded first so the built-in cube always gets asset id 0
        reg.read_mesh_from_path(Path::new(BUILTIN_CUBE_PATH))?;

        Ok(reg)
    }

    fn add_to_cache(&mut self, asset: RawAsset) {
        self.cache.insert(asset.get_path_boxed().clone(), asset);
    }

    fn read_raw_from_path(&mut self, path: &Path) -> Result<&RawAsset, asset_error::AssetError> {
        if !self.cache.contains_key(path) {
            let asset = RawAsset::from_path(path.to_path_buf())?;
            self.add_to_cache(asset);
        }
        Ok(self.cache.get(path).unwrap())
    }

    /// Loads (and caches) the mesh at `path`, returning a stable [`AssetId`]
    /// that [`Self::get_mesh`] can later resolve back to the parsed
    /// [`MeshAsset`]. Repeated calls with the same path return the same id
    /// without re-parsing.
    pub(crate) fn read_mesh_from_path(
        &mut self,
        path: &Path,
    ) -> Result<AssetId, asset_error::AssetError> {
        if let Some(&id) = self.mesh_ids.get(path) {
            return Ok(id);
        }

        let raw = RawAsset::from_path(path.to_path_buf())?;
        let mesh = MeshAsset::from_raw(&raw)?;

        let id = self.next_mesh_id;
        self.next_mesh_id += 1;

        self.mesh_ids
            .insert(path.to_path_buf().into_boxed_path(), id);
        self.meshes.insert(id, mesh);
        self.add_to_cache(raw);

        Ok(id)
    }

    #[inline]
    pub(crate) fn get_mesh(&self, id: AssetId) -> Option<&MeshAsset> {
        self.meshes.get(&id)
    }
}
