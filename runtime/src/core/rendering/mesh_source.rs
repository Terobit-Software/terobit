use std::collections::HashMap;

use crate::assets::AssetReference;

/// default values:
/// 0 = cube
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MeshSourceId(pub usize);

impl MeshSourceId {
    pub fn from_reference (asset: AssetReference) -> Self {
        // TODO: actually make this work lol
        Self(0)
    }

    pub const CUBE: Self = Self(0);
    // etc.
}

impl From<MeshSourceId> for usize {
    fn from(id: MeshSourceId) -> usize { id.0 }
}

impl From<usize> for MeshSourceId {
    fn from(n: usize) -> Self { Self(n) }
}

pub(crate) struct MeshSource {
    id: MeshSourceId
}

pub(crate) struct MeshSourceHandler {
    meshes: HashMap<MeshSourceId, MeshSource>
}

impl MeshSource {
    pub fn cube () -> Self {
        MeshSource { 
            id: MeshSourceId(0)
        }
    }

    pub fn from_id (id: MeshSourceId) -> Self {
        MeshSource { id }
    }

    pub(crate) fn get_id (&self) -> MeshSourceId {
        self.id
    }
}