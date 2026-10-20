use crate::pack::{IndexV1, ManifestV1};

pub struct TbpkSource {
    manifest: ManifestV1,
    index: IndexV1,
}

impl TbpkSource {
    pub fn new(manifest: ManifestV1, index: IndexV1) -> Self {
        Self { manifest, index }
    }
}
