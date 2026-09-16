use crate::assets::AssetId;

/// References a mesh loaded into the runtime's asset registry by its
/// [`AssetId`], rather than owning the parsed mesh data directly.
#[derive(bevy_ecs::component::Component)]
pub struct Mesh {
    asset_id: AssetId,
}

impl Mesh {
    pub fn new(asset_id: AssetId) -> Self {
        Self { asset_id }
    }

    #[inline]
    pub fn asset_id(&self) -> AssetId {
        self.asset_id
    }
}
