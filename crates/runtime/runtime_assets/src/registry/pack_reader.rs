use bevy::asset::io::{AssetReader, AssetReaderError, Reader, VecReader};
use std::path::Path;

use crate::registry::PackRegistry;

/// the `pack://` asset source. its paths are pack paths, like `meshes/mesh.obj`
pub(crate) struct PackAssetReader(pub(crate) PackRegistry);

impl AssetReader for PackAssetReader {
    async fn read<'a>(&'a self, path: &'a Path) -> Result<impl Reader + 'a, AssetReaderError> {
        let full_path = self
            .0
            .full_path(path)
            .ok_or_else(|| AssetReaderError::NotFound(path.into()))?;
        Ok(VecReader::new(std::fs::read(full_path)?))
    }

    async fn read_meta<'a>(&'a self, path: &'a Path) -> Result<impl Reader + 'a, AssetReaderError> {
        Err::<VecReader, _>(AssetReaderError::NotFound(path.into())) // no .meta files in packs
    }
    // read_directory / is_directory: answer from the index (folder loading)
    async fn read_directory<'a>(
        &'a self,
        path: &'a std::path::Path,
    ) -> Result<Box<bevy::asset::io::PathStream>, AssetReaderError> {
        Err(AssetReaderError::NotFound(path.into()))
    }

    async fn is_directory<'a>(
        &'a self,
        path: &'a std::path::Path,
    ) -> Result<bool, AssetReaderError> {
        // TODO: answer from the index once folder loading is supported
        Err(AssetReaderError::NotFound(path.into()))
    }
}
