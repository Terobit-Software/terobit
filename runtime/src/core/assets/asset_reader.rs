use crate::assets::AssetReference;

/// reads assets from disk (placeholder, not actually implemented yet)
pub struct AssetReader {

}

impl AssetReader {
    /// creates a new asset reader
    pub fn new () -> Self {
        AssetReader { }
    }

    /// reads an asset and returns a reference (placeholder — returns a dummy reference)
    pub fn read (&self) -> AssetReference {
        AssetReference::new()
    }
}