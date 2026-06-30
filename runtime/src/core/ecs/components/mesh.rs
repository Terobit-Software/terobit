use crate::rendering::MeshSourceId;

/// references a mesh to be rendered at this entity's transform
#[derive(bevy_ecs::component::Component, Clone)]
pub struct Mesh {
    id: MeshSourceId,
}

impl Mesh {
    /// creates a cube mesh component (the only built-in mesh right now)
    pub fn cube() -> Self {
        Mesh { id: MeshSourceId::CUBE }
    }

    /// creates a mesh component from a pre-registered mesh id.
    /// use `RuntimeContext::load_mesh` to get an id from a file
    pub(crate) fn with_source_id(id: MeshSourceId) -> Self {
        Mesh { id }
    }

    pub(crate) fn get_id(&self) -> &MeshSourceId {
        &self.id
    }
}
