use crate::assets::AssetReference;

pub struct AssetReader {

}

impl AssetReader {
    pub fn new () -> Self {
        AssetReader { }
    }

    pub fn read (&self) -> AssetReference {
        AssetReference::new()
    }
}