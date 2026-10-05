use std::path::Path;

pub(crate) enum ManifestAccessLevel {
    Free,
    Licensed,
}

pub(crate) struct ManifestSource {
    realm_id: u64,
    owner_id: u64,
    pack_version: u64,
    access: ManifestAccessLevel,
}

impl ManifestSource {
    pub(crate) fn from_path(path: Box<Path>) -> Result<Self, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        let arena = toml_spanner::Arena::new();
        let doc = toml_spanner::parse(&content, &arena);

        
    }
}
