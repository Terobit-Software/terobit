use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::assets::asset_error;
use crate::{MeshAsset, RawAsset};

pub(crate) type AssetId = usize;

pub(crate) struct AssetRegistry {
    cache: HashMap<Box<Path>, RawAsset>,
}

impl AssetRegistry {
    pub(crate) fn new() -> Result<Self, asset_error::AssetError> {
        let mut reg = Self {
            cache: HashMap::new(),
        };

        reg.add_to_cache(RawAsset::from_path(PathBuf::from("bi.cubeobj"))?);

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

    pub(crate) fn read_mesh_from_path(
        &mut self,
        path: &Path,
    ) -> Result<MeshAsset, asset_error::AssetError> {
        Ok(MeshAsset::from_path(path))
    }
}
