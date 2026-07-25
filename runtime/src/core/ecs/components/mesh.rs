use crate::rendering::MeshSourceId;

/// references a mesh to be rendered at this entity's transform
#[derive(bevy_ecs::component::Component, Clone)]
pub struct Mesh {
    id: MeshSourceId,
}

impl Mesh {
    /// creates a cube mesh component
    pub fn cube() -> Self {
        Mesh {
            id: MeshSourceId::CUBE,
        }
    }

    /// creates a flat plane mesh component (unit quad in the xz plane, facing +y).
    /// scale it via `Transform` to make a floor
    pub fn plane() -> Self {
        Mesh {
            id: MeshSourceId::PLANE,
        }
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
