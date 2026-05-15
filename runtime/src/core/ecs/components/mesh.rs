use crate::{assets::AssetReference, rendering::{MeshSource, MeshSourceId}};
#[derive(bevy_ecs::component::Component)]
pub struct Mesh {
    id: MeshSourceId
}

impl Mesh {
    pub fn from_asset (asset: AssetReference) -> Self {
        Mesh {
            id: MeshSourceId::from_reference(asset)
        }
    }

    pub fn cube () -> Self {
        Mesh {
            id: MeshSourceId::CUBE
        }
    }

    pub(crate) fn get_id (&self) -> &MeshSourceId {
        &self.id
    }
}