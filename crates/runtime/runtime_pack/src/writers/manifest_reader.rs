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

#[cfg(test)]
mod struct_tests {
    use super::*;
    use std::path::{Path, PathBuf};

    fn example_manifest() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../tests/examples/source_to_tbpk/basic1/manifest.toml")
    }

    fn parse(name: &str, text: &str) -> Result<ManifestSource, String> {
        let path = std::env::temp_dir().join(format!(
            "terobit_manifest_{}_{}.toml",
            std::process::id(),
            name
        ));
        std::fs::write(&path, text).unwrap();
        let result = ManifestSource::from_path(&path);
        std::fs::remove_file(&path).unwrap();
        result
    }

    #[test]
    fn reads_example_manifest() {
        let manifest = ManifestSource::from_path(&example_manifest()).unwrap();

        assert_eq!(manifest.realm_id, 1);
        assert_eq!(manifest.owner_id, 1);
        assert_eq!(manifest.pack_version, 1);
        assert!(matches!(manifest.access, ManifestAccessLevel::Free));
        assert_eq!(
            manifest.assets.get("self:cube").map(String::as_str),
            Some("meshes/mesh.obj")
        );
        assert_eq!(
            manifest
                .assets
                .get("self:cube.material")
                .map(String::as_str),
            Some("meshes/mesh.mtl")
        );
        assert_eq!(manifest.assets.len(), 2);
        assert_eq!(
            manifest.client.assets.get("self:car").map(String::as_str),
            Some("images/car.gif")
        );
        assert_eq!(manifest.client.assets.len(), 1);
        assert_eq!(
            manifest.server.assets.get("self:music").map(String::as_str),
            Some("audio/music.mp3")
        );
        assert_eq!(manifest.server.assets.len(), 1);
    }

    #[test]
    fn asset_tables_default_to_empty() {
        let manifest = parse(
            "minimal",
            "realm_id = 2\nowner_id = 3\npack_version = 4\naccess = \"licensed\"\n",
        )
        .unwrap();

        assert_eq!(manifest.realm_id, 2);
        assert_eq!(manifest.owner_id, 3);
        assert_eq!(manifest.pack_version, 4);
        assert!(matches!(manifest.access, ManifestAccessLevel::Licensed));
        assert!(manifest.assets.is_empty());
        assert!(manifest.client.assets.is_empty());
        assert!(manifest.server.assets.is_empty());
    }

    #[test]
    fn empty_side_table_is_accepted() {
        let manifest = parse(
            "empty_side",
            "realm_id = 1\nowner_id = 1\npack_version = 1\naccess = \"free\"\n[client]\n",
        )
        .unwrap();

        assert!(manifest.client.assets.is_empty());
    }

    #[test]
    fn rejects_unknown_top_level_field() {
        assert!(
            parse(
                "unknown_field",
                "realm_id = 1\nowner_id = 1\npack_version = 1\naccess = \"free\"\nextra = 1\n",
            )
            .is_err()
        );
    }

    #[test]
    fn rejects_unknown_side_field() {
        assert!(
            parse(
                "unknown_side_field",
                "realm_id = 1\nowner_id = 1\npack_version = 1\naccess = \"free\"\n[client]\nextra = 1\n",
            )
            .is_err()
        );
    }

    #[test]
    fn rejects_unknown_access_level() {
        assert!(
            parse(
                "bad_access",
                "realm_id = 1\nowner_id = 1\npack_version = 1\naccess = \"paid\"\n",
            )
            .is_err()
        );
    }

    #[test]
    fn access_level_is_lowercase_only() {
        assert!(
            parse(
                "uppercase_access",
                "realm_id = 1\nowner_id = 1\npack_version = 1\naccess = \"Free\"\n",
            )
            .is_err()
        );
    }

    #[test]
    fn rejects_missing_required_field() {
        assert!(
            parse(
                "missing_field",
                "realm_id = 1\nowner_id = 1\naccess = \"free\"\n"
            )
            .is_err()
        );
    }

    #[test]
    fn rejects_negative_id() {
        assert!(
            parse(
                "negative_id",
                "realm_id = -1\nowner_id = 1\npack_version = 1\naccess = \"free\"\n",
            )
            .is_err()
        );
    }

    #[test]
    fn rejects_invalid_toml() {
        assert!(parse("invalid_toml", "realm_id = \n").is_err());
    }

    #[test]
    fn missing_file_is_error() {
        let path = std::env::temp_dir().join("terobit_manifest_does_not_exist.toml");
        assert!(ManifestSource::from_path(&path).is_err());
    }
}
