use crate::assets::AssetReference;

pub(crate) type MeshSourceId = u32;

pub(crate) struct MeshSource {
    id: MeshSourceId
}


impl MeshSource {
    pub fn from_reference (reference: AssetReference) -> Self {
        MeshSource { 
            id: 0
        }
    }

    pub fn cube () -> Self {
        MeshSource { 
            id: 0
        }
    }

    pub(crate) fn get_id (&self) -> MeshSourceId {
        self.id
    }
}