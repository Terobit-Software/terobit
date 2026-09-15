use crate::assets::MeshAsset;

#[derive(bevy_ecs::component::Component)]
pub struct Mesh {
    asset: MeshAsset,
}

impl Mesh {
    pub fn new(asset: MeshAsset) -> Self {
        Self { asset }
    }
    
}