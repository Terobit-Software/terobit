use std::path::PathBuf;

/// the category of asset being referenced
pub enum AssetReferenceType {
    Mesh,
}

/// a reference to an asset file on disk, passed to `RuntimeContext::load_mesh`
pub struct AssetReference {
    /// path to the asset file
    pub path: PathBuf,
    pub asset_type: AssetReferenceType,
}

impl AssetReference {
    /// creates a reference to a mesh asset at the given path
    pub fn mesh(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            asset_type: AssetReferenceType::Mesh,
        }
    }
}
