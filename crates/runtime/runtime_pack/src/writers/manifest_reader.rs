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

pub(crate) struct ManifestReader {}
