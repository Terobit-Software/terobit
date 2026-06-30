use crate::{assets::AssetReference, rendering::{MeshSource, MeshSourceId}};
/// references a mesh to be rendered at this entity's transform
#[derive(bevy_ecs::component::Component)]
pub struct Mesh {
    id: MeshSourceId
}

impl Mesh {
    /// creates a mesh component from an asset file reference
    pub fn from_asset (asset: AssetReference) -> Self {
        Mesh {
            id: MeshSourceId::from_reference(asset)
        }
    }

    /// creates a cube mesh component (the only built-in mesh right now)
    pub fn cube () -> Self {
        Mesh {
            id: MeshSourceId::CUBE
        }
    }

    pub(crate) fn get_id (&self) -> &MeshSourceId {
        &self.id
    }
}