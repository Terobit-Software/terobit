use crate::pack::AssetHandle;
use crate::registry::TbpkSource;
use bevy::ecs::resource::Resource;
use snafu::OptionExt;
use snafu::ResultExt;
use snafu::Snafu;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::RwLock;

#[derive(Resource, Clone, Default)]
pub struct PackRegistry(Arc<RwLock<RegistryInner>>);

pub enum RootLocation {
    Developer { path: Box<Path> },
    Pack { id: u64 },
}

#[derive(Default)]
pub struct RegistryInner {
    /// <RealmId, TbpkSource>
    sources: HashMap<u64, TbpkSource>,
}

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum RegistryError {
    #[snafu(display("failed to get read access to registry with handle `{}`", handle))]
    FailedReadAccess { handle: AssetHandle },

    #[snafu(display("could not find realmid `{}` in sources", id))]
    FailedGetRealmFromId { id: u64 },

    #[snafu(display("could not resolve handle `{}` to path", handle))]
    FailedToResolveHandle { handle: AssetHandle },

    #[snafu(display("failed to get read access to registry"))]
    FailedReadLock,

    #[snafu(display("could not read from path `{}`", path.display()))]
    FailedToReadAssetFromPath {
        path: PathBuf,
        source: std::io::Error,
    },

    #[snafu(display("reading `{}` from pack `{}` is not supported yet", path.display(), id))]
    PackReadUnsupported { id: u64, path: PathBuf },
}

impl PackRegistry {
    pub(crate) fn resolve(&self, handle: &AssetHandle) -> Result<Box<Path>, RegistryError> {
        let read = self.0.read().map_err(|_| {
            FailedReadAccessSnafu {
                handle: handle.clone(),
            }
            .build()
        })?;

        let source = read
            .sources
            .get(&0u64)
            .context(FailedGetRealmFromIdSnafu { id: 0u64 })?;

        let path = source
            .resolve_handle_to_path(handle)
            .context(FailedToResolveHandleSnafu {
                handle: handle.clone(),
            })?;

        Ok(path.clone())
    }

    pub fn read_bytes(&self, path: &Path) -> Result<Vec<u8>, RegistryError> {
        // needs to be updated to async some time
        let full_path = {
            let read = self.0.read().map_err(|_| FailedReadLockSnafu.build())?;

            let source = read
                .sources
                .get(&0u64)
                .context(FailedGetRealmFromIdSnafu { id: 0u64 })?;

            match source.get_root() {
                RootLocation::Developer { path: root } => root.join(path),
                RootLocation::Pack { id } => {
                    return PackReadUnsupportedSnafu { id: *id, path }.fail();
                }
            }
        };

        tracing::info!("Reading from path {}", full_path.display());

        std::fs::read(&full_path).context(FailedToReadAssetFromPathSnafu { path: full_path })
    }
}
