
/// the kind of asset being referenced
pub enum AssetReferenceType {
    MESH
}

/// a reference to an asset on disk (placeholder, not actually loading anything yet)
pub struct AssetReference {
    asset_type: AssetReferenceType
}

impl AssetReference {
    /// creates a new asset reference (placeholder — always returns a mesh type for now)
    pub fn new () -> Self {
        // TODO: actually make it work yk

        AssetReference {
            asset_type: AssetReferenceType::MESH
        }
    }
}