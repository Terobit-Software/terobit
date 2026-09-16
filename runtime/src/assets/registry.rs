use std::collections::HashMap;
use std::path::Path;

use crate::RawAsset;
use crate::assets::asset_error;

pub(crate) type AssetId = usize;

pub(crate) struct AssetRegistry {
    cache: HashMap<Box<Path>, RawAsset>,
}

impl AssetRegistry {
    pub(crate) fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    fn add_to_cache(&mut self, asset: RawAsset) {
        self.cache.insert(asset.get_path_boxed().clone(), asset);
    }

    pub(crate) fn read_from_path(
        &mut self,
        path: &Path,
    ) -> Result<&RawAsset, asset_error::AssetError> {
        if !self.cache.contains_key(path) {
            let asset = RawAsset::from_path(path.to_path_buf())?;
            self.add_to_cache(asset);
        }
        Ok(self.cache.get(path).unwrap())
    }
}
