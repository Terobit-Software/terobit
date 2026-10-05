use crate::readers::PackReader;
use bevy::prelude::*;
use std::path::Path;
use tracing::info;

pub struct RuntimePackReaderPlugin;

#[derive(Event)]
pub struct LoadPackFileEvent {
    pub path: Box<Path>,
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
    fn scan_pack(e: On<LoadPackFileEvent>, mut commands: Commands, config: Res<RuntimePackConfig>) {
        let path = e.event().path.as_ref();

        PackReader::read_pack(&mut commands, path, &config);
    }
}

impl Plugin for RuntimePackReaderPlugin {
    fn build(&self, app: &mut App) {
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
}
