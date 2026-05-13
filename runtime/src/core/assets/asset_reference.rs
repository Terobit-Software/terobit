
pub enum AssetReferenceType {
    MESH
}

pub struct AssetReference {
    asset_type: AssetReferenceType
}

impl AssetReference {
    pub fn new () -> Self {
        // TODO: actually make it work yk
        
        AssetReference { 
            asset_type: AssetReferenceType::MESH
        }
    }
}