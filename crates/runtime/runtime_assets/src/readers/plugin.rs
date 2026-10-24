use crate::PackReader;
use crate::pack::Target;
use crate::registry::{PackAssetReader, PackRegistry};
use bevy::{asset::io::AssetSourceBuilder, prelude::*};
use std::path::Path;
use tracing::info;

/// reads packs into the `PackRegistry` and serves them as the `pack://` asset source. add it
/// before `AssetPlugin`
pub struct RuntimePackReaderPlugin;

#[derive(Event)]
pub struct LoadPackFileEvent {
    pub path: Box<Path>,
    /// the side reading the pack, only the shared entries and this side's are loaded
    pub target: Target,
}

#[derive(Resource, Default)]
pub struct RuntimePackConfig {
    allow_dir_read: bool,
}

impl RuntimePackConfig {
    pub fn get_allow_dir_read(&self) -> &bool {
        &self.allow_dir_read
    }
    pub fn set_allow_dir_read(&mut self, value: bool) {
        self.allow_dir_read = value;
        info!("RuntimePack allow_dir_read has been set to {}", value);
    }
}

impl RuntimePackReaderPlugin {
    fn scan_pack(
        e: On<LoadPackFileEvent>,
        config: Res<RuntimePackConfig>,
        registry: Res<PackRegistry>,
    ) {
        let LoadPackFileEvent { path, target } = e.event();

        match PackReader::read_pack(path, *target, &config, &registry) {
            Err(e) => tracing::error!(
                "Failed to read pack `{}`, nothing from it was loaded: {}",
                path.display(),
                snafu::Report::from_error(&e)
            ),
            Ok(()) => tracing::info!("Read pack `{}` successfully", path.display()),
        }
    }
}

impl Plugin for RuntimePackReaderPlugin {
    fn build(&self, app: &mut App) {
        let registry = PackRegistry::default();
        app.insert_resource(registry.clone());
        app.register_asset_source(
            "pack",
            AssetSourceBuilder::new(move || Box::new(PackAssetReader(registry.clone()))),
        );
        app.add_observer(Self::scan_pack);
        app.init_resource::<RuntimePackConfig>();
    }
}

#[cfg(test)]
mod struct_tests {
    use super::*;

    #[test]
    fn config_defaults_to_no_dir_read() {
        assert!(!*RuntimePackConfig::default().get_allow_dir_read());
    }

    #[test]
    fn config_set_allow_dir_read() {
        let mut config = RuntimePackConfig::default();

        config.set_allow_dir_read(true);
        assert!(*config.get_allow_dir_read());

        config.set_allow_dir_read(false);
        assert!(!*config.get_allow_dir_read());
    }
}

#[cfg(test)]
mod plugin_tests {
    use super::*;
    use crate::pack::AssetHandle;
    use crate::registry::{PackAssets, UnknownHandle};
    use bevy::asset::io::Reader;
    use bevy::asset::{AssetLoader, LoadContext, LoadState};
    use bevy::ecs::system::RunSystemOnce;
    use bevy::reflect::TypePath;
    use std::path::PathBuf;

    fn app() -> App {
        let mut app = App::new();
        app.add_plugins(RuntimePackReaderPlugin);
        app
    }

    fn temp_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("terobit_pack_{}_{}", std::process::id(), name))
    }

    fn load(app: &mut App, path: PathBuf) {
        app.world_mut().trigger(LoadPackFileEvent {
            path: path.into_boxed_path(),
            target: Target::Client,
        });
        app.update();
    }

    #[test]
    fn adds_default_config() {
        let app = app();

        let config = app.world().get_resource::<RuntimePackConfig>();
        assert!(config.is_some());
        assert!(!*config.unwrap().get_allow_dir_read());
    }

    #[test]
    fn keeps_existing_config() {
        let mut app = App::new();
        let mut config = RuntimePackConfig::default();
        config.set_allow_dir_read(true);
        app.insert_resource(config);
        app.add_plugins(RuntimePackReaderPlugin);

        assert!(
            *app.world()
                .resource::<RuntimePackConfig>()
                .get_allow_dir_read()
        );
    }

    #[test]
    fn handles_missing_file() {
        let mut app = app();
        load(&mut app, temp_path("missing.tbpk"));
    }

    #[test]
    fn handles_directory_without_dir_read() {
        let mut app = app();
        load(&mut app, std::env::temp_dir());
    }

    #[test]
    fn handles_directory_with_dir_read() {
        let mut app = app();
        app.world_mut()
            .resource_mut::<RuntimePackConfig>()
            .set_allow_dir_read(true);
        load(&mut app, std::env::temp_dir());
    }

    #[test]
    fn handles_file_shorter_than_header() {
        let path = temp_path("short.tbpk");
        std::fs::write(&path, [0u8; 4]).unwrap();

        let mut app = app();
        load(&mut app, path.clone());

        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn handles_file_longer_than_header() {
        let path = temp_path("long.tbpk");
        std::fs::write(&path, [0u8; 256]).unwrap();

        let mut app = app();
        load(&mut app, path.clone());

        std::fs::remove_file(&path).unwrap();
    }

    #[derive(Asset, TypePath)]
    struct Bytes(Vec<u8>);

    #[derive(TypePath)]
    struct BytesLoader;

    impl AssetLoader for BytesLoader {
        type Asset = Bytes;
        type Settings = ();
        type Error = std::io::Error;

        async fn load(
            &self,
            reader: &mut dyn Reader,
            _: &(),
            _: &mut LoadContext<'_>,
        ) -> Result<Bytes, Self::Error> {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            Ok(Bytes(bytes))
        }

        fn extensions(&self) -> &[&str] {
            &["obj", "mp3"]
        }
    }

    fn basic1() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../tests/examples/source_to_tbpk/basic1")
    }

    /// a client app with basic1 loaded as a dev directory
    fn dev_app() -> App {
        let mut app = App::new();
        app.add_plugins((
            TaskPoolPlugin::default(),
            RuntimePackReaderPlugin,
            AssetPlugin::default(),
        ))
        .init_asset::<Bytes>()
        .register_asset_loader(BytesLoader);
        app.finish();
        app.cleanup();

        app.world_mut()
            .resource_mut::<RuntimePackConfig>()
            .set_allow_dir_read(true);
        load(&mut app, basic1());
        app
    }

    /// runs the app until `handle` loads, `None` if it fails
    fn wait_for(app: &mut App, handle: &Handle<Bytes>) -> Option<Vec<u8>> {
        for _ in 0..10_000 {
            app.update();
            match app.world().resource::<AssetServer>().load_state(handle) {
                LoadState::Loaded => {
                    let assets = app.world().resource::<Assets<Bytes>>();
                    return assets.get(handle).map(|bytes| bytes.0.clone());
                }
                LoadState::Failed(_) => return None,
                _ => std::thread::sleep(std::time::Duration::from_millis(1)),
            }
        }
        panic!("asset didn't finish loading");
    }

    fn load_by_handle(app: &mut App, id: &str) -> Result<Handle<Bytes>, UnknownHandle> {
        let handle: AssetHandle = id.parse().unwrap();
        app.world_mut()
            .run_system_once(move |assets: PackAssets| assets.load(&handle))
            .unwrap()
    }

    #[test]
    fn loads_dev_directory_assets_by_handle() {
        let mut app = dev_app();

        let handle = load_by_handle(&mut app, "self:cube").unwrap();

        assert_eq!(
            wait_for(&mut app, &handle),
            Some(std::fs::read(basic1().join("meshes/mesh.obj")).unwrap())
        );
    }

    #[test]
    fn client_cant_load_server_assets() {
        let mut app = dev_app();

        assert!(load_by_handle(&mut app, "self:music").is_err());

        let by_path = app
            .world()
            .resource::<AssetServer>()
            .load("pack://audio/music.mp3");
        assert_eq!(wait_for(&mut app, &by_path), None);
    }
}
