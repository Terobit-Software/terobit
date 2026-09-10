use crate::RawAsset;
use crate::assets::AssetError;
use std::path::{Path, PathBuf};

pub(crate) struct ShaderAsset {
    data: String,
    // used box path for less memory footprint
    path: Box<Path>,
}

impl ShaderAsset {
    pub(crate) fn from_raw(raw: RawAsset) -> Self {
        Self {
            data: String::from_utf8_lossy(raw.get_data()).into_owned(),
            path: raw.get_path().to_owned().into_boxed_path(),
        }
    }

    pub(crate) fn from_path(path: PathBuf) -> Result<Self, AssetError> {
        Ok(Self::from_raw(RawAsset::from_path(path)?))
    }

    pub(crate) fn get_data(&self) -> &str {
        &self.data
    }

    pub(crate) fn get_path(&self) -> &Path {
        &self.path
    }
}
