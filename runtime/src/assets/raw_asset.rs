use std::path::{Path, PathBuf};
use tracing::{info, warn};

use crate::AssetError;

pub(crate) struct RawAsset {
    data: Vec<u8>,
    // used box path for less memory footprint
    path: Box<Path>,
}

impl RawAsset {
    pub(crate) fn from_path(path: PathBuf) -> Result<Self, AssetError> {
        match std::fs::read(path.clone()) {
            Ok(v) => {
                info!("Read file at {} successfully.", path.to_string_lossy());
                Ok(Self {
                    data: v,
                    path: path.into_boxed_path(),
                })
            }
            Err(v) => {
                warn!("Failed to read file at {}.", path.to_string_lossy());
                Err(AssetError::FailedToReadFile(v))
            }
        }
    }

    #[inline]
    pub(crate) fn get_data(&self) -> &[u8] {
        &self.data
    }

    #[inline]
    pub(crate) fn get_path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn get_path_boxed(&self) -> &Box<Path> {
        &self.path
    }
}
