use std::path::{Path, PathBuf};
use tracing::{info, warn};

use crate::AssetError;
use crate::assets::registry::{AssetId, AssetRegistry};

/// Path used to request the built-in cube mesh instead of reading a file from disk.
pub(crate) const BUILTIN_CUBE_PATH: &str = "bi.cubeobj";

/// Raw bytes read from disk (or synthesized for a built-in asset). Owned
/// exclusively by the [`AssetRegistry`]; nothing outside the registry holds
/// this data directly.
pub(crate) struct RawAssetData {
    /// if the file is builtin, its data will be its id, not its content
    is_builtin: bool,
    data: Vec<u8>,
    // used box path for less memory footprint
    /// empty if the file is builtin
    path: Box<Path>,
}

impl RawAssetData {
    pub(crate) fn read(path: &Path) -> Result<Self, AssetError> {
        if path == Path::new(BUILTIN_CUBE_PATH) {
            return Ok(Self {
                is_builtin: true,
                data: Vec::from(BUILTIN_CUBE_PATH),
                path: PathBuf::new().into_boxed_path(),
            });
        }

        match std::fs::read(path) {
            Ok(data) => {
                info!("Read file at {} successfully.", path.to_string_lossy());
                Ok(Self {
                    is_builtin: false,
                    data,
                    path: path.to_path_buf().into_boxed_path(),
                })
            }
            Err(err) => {
                warn!("Failed to read file at {}.", path.to_string_lossy());
                Err(AssetError::FailedToReadFile(err))
            }
        }
    }

    #[inline]
    pub(crate) fn data(&self) -> &[u8] {
        &self.data
    }

    #[inline]
    pub(crate) fn is_builtin(&self) -> bool {
        self.is_builtin
    }

    #[inline]
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}

/// A handle to raw asset bytes stored in the [`AssetRegistry`]. Holds only
/// an [`AssetId`] — every getter takes the registry and looks the data up
/// there.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RawAsset {
    id: AssetId,
}

impl RawAsset {
    #[inline]
    pub(crate) fn new(id: AssetId) -> Self {
        Self { id }
    }

    #[inline]
    pub(crate) fn id(&self) -> AssetId {
        self.id
    }

    #[inline]
    pub(crate) fn get_data<'a>(&self, registry: &'a AssetRegistry) -> &'a [u8] {
        registry.raw_data(self.id)
    }

    #[inline]
    pub(crate) fn is_builtin(&self, registry: &AssetRegistry) -> bool {
        registry.raw_is_builtin(self.id)
    }

    #[inline]
    pub(crate) fn get_path<'a>(&self, registry: &'a AssetRegistry) -> &'a Path {
        registry.raw_path(self.id)
    }
}
