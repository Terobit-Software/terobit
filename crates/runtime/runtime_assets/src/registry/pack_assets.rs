use bevy::asset::{Asset, AssetServer, Handle};
use bevy::ecs::system::{Res, SystemParam};
use snafu::{OptionExt, Snafu};

use crate::pack::AssetHandle;
use crate::registry::PackRegistry;

/// a handle that no loaded pack declares for this side
#[derive(Debug, Snafu)]
#[snafu(display(
    "no loaded pack declares asset `{handle}`, check the manifest's [assets] tables and that its pack was read first"
))]
pub struct UnknownHandle {
    pub handle: AssetHandle,
}

/// loads pack assets by handle, use in systems in place of `Res<AssetServer>`
#[derive(SystemParam)]
pub struct PackAssets<'w> {
    server: Res<'w, AssetServer>,
    registry: Res<'w, PackRegistry>,
}

impl PackAssets<'_> {
    pub fn load<A: Asset>(&self, h: &AssetHandle) -> Result<Handle<A>, UnknownHandle> {
        let path = self
            .registry
            .resolve(h)
            .context(UnknownHandleSnafu { handle: h.clone() })?;
        Ok(self.server.load(format!("pack://{path}")))
    }
}
