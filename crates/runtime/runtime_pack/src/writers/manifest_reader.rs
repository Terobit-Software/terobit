pub(crate) enum ManifestAcessLevel {
    Free,
    Licensed,
}

pub(crate) struct ManifestSource {
    realm_id: u64,
    owner_id: u64,
    pack_version: u64,
    access: ManifestAcessLevel,
}

pub(crate) struct ManifestReader {}
