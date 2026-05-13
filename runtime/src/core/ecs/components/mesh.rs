use crate::{assets::AssetReference, rendering::MeshSource};
#[derive(bevy_ecs::component::Component)]
pub struct Mesh {
    source: MeshSource
}

impl Mesh {
    pub fn from_asset (asset: AssetReference) -> Self {
        Mesh {
            source: MeshSource::from_reference(asset)
        }
    }

    pub fn cube () -> Self {
        Mesh {
            source: MeshSource::cube()
        }
    }
}