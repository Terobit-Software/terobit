use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ManifestAccessLevel {
    Free,
    Licensed,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManifestSource {
    realm_id: u64,
    owner_id: u64,
    pack_version: u64,
    access: ManifestAccessLevel,
    #[serde(default)]
    assets: BTreeMap<String, String>,
    #[serde(default)]
    client: SideAssets,
    #[serde(default)]
    server: SideAssets,
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub(crate) struct SideAssets {
    #[serde(default)]
    // btree instead of hashmap for determinism in builds
    assets: BTreeMap<String, String>,
}

impl ManifestSource {
    pub fn from_path(path: &std::path::Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        toml::from_str::<Self>(&text).map_err(|e| e.to_string())
    }
}
