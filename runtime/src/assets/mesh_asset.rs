use std::path::Path;

use crate::assets::AssetId;

pub(crate) struct MeshAsset {
    verts: Vec<Vertex>,
}

impl MeshAsset {
    // TODO: impl this
    pub fn from_path(path: &Path) -> Self {
        Self { id: 0 }
    }
}
