use bevy::asset::io::{AssetReader, AssetReaderError, Reader, VecReader};
use bevy::asset::{Asset, AssetServer};
use std::path::Path;

use crate::pack::HandleError;
use crate::registry::PackRegistry;

struct PackAssetReader(PackRegistry);

impl AssetReader for PackAssetReader {
    async fn read<'a>(&'a self, path: &'a Path) -> Result<impl Reader + 'a, AssetReaderError> {
        let reg = &self.0;
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
