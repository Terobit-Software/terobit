use std::path::{Path, PathBuf};
use tracing::{info, warn};

use crate::AssetError;

/// Path used to request the built-in cube mesh instead of reading a file from disk.
pub(crate) const BUILTIN_CUBE_PATH: &str = "bi.cubeobj";

pub(crate) struct RawAssetData {
    /// if the file is builtin, its data will be its id, not its content
    is_builtin: bool,
    data: Vec<u8>,
}

pub(crate) struct RawAsset {
    data: RawAssetData,
    // used box path for less memory footprint
    /// empty if the file is builtin
    path: Box<Path>,
}

impl RawAsset {
    pub(crate) fn from_path(path: PathBuf) -> Result<Self, AssetError> {
        if path == PathBuf::from(BUILTIN_CUBE_PATH) {
            return Ok(Self {
                data: RawAssetData {
                    is_builtin: true,
                    data: Vec::from(BUILTIN_CUBE_PATH),
                },
                path: PathBuf::new().into_boxed_path(),
            });
        }

        match std::fs::read(path.clone()) {
            Ok(v) => {
                info!("Read file at {} successfully.", path.to_string_lossy());
                Ok(Self {
                    data: RawAssetData {
                        is_builtin: false,
                        data: v,
                    },
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
        &self.data.data
    }

    #[inline]
    pub(crate) fn is_builtin(&self) -> bool {
        self.data.is_builtin
    }

    #[inline]
    pub(crate) fn get_path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn get_path_boxed(&self) -> &Box<Path> {
        &self.path
    }
}
