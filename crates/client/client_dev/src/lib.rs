use bevy::prelude::*;
use runtime_assets::pack::readers::RuntimePackConfig;
use std::path::PathBuf;
use tracing::warn;

pub struct ClientDevPlugin;

#[derive(Resource)]
pub struct DeveloperModeStatus {
    value: bool,
}

impl DeveloperModeStatus {
    pub fn get_value(&self) -> &bool {
        &self.value
    }
}

impl Plugin for ClientDevPlugin {
    fn build(&self, app: &mut App) {
        let (is_dev, pack_file) = {
            let flags = app
                .world()
                .resource::<runtime_run_flags::RuntimeFlags>()
                .get_flags();

            let pack_file = flags.get("--pack-file").map(|values| {
                PathBuf::from(values.first().expect("No pack file given in --pack-file"))
                    .into_boxed_path()
            });

            (flags.contains_key("--dev"), pack_file)
        };

        if is_dev {
            tracing::info!(
                "\n\n--------------------------------\nClient running in DEVELOPER mode\n--------------------------------\n"
            );
            app.world_mut()
                .insert_resource(DeveloperModeStatus { value: true });

            if let Some(mut config) = app.world_mut().get_resource_mut::<RuntimePackConfig>() {
                config.set_allow_dir_read(true);
            } else {
                warn!(
                    "Failed to set RuntimePackConfig allow_dir_read to true, config does not exist"
                )
            }

            if let Some(path) = pack_file {
                app.world_mut()
                    .trigger(runtime_assets::pack::readers::LoadPackFileEvent { path });
            } else {
                warn!("Failed to fire LoadPackFileEvent, event does not exist")
            }

            return;
        }

        tracing::info!("Client running in normal mode");
        app.world_mut()
            .insert_resource(DeveloperModeStatus { value: false });
    }
}

#[cfg(test)]
mod plugin_tests {
    use super::*;
    use runtime_run_flags::*;

    fn dev_status(args: &[&str]) -> bool {
        let mut app = App::new();
        app.insert_resource(RuntimeFlags::from_args(
            args.iter().map(|arg| arg.to_string()),
        ));
        app.add_plugins(ClientDevPlugin);

        *app.world().resource::<DeveloperModeStatus>().get_value()
    }

    #[test]
    fn sets_dev_status_true() {
        assert!(dev_status(&["--dev"]));
    }

    #[test]
    fn sets_dev_status_false() {
        assert!(!dev_status(&[]));
    }

    #[derive(Resource, Default)]
    struct LoadedPacks(Vec<PathBuf>);

    /// builds the app with a `RuntimePackConfig` and records every `LoadPackFileEvent`
    fn app_with_pack_config(args: &[&str]) -> App {
        let mut app = App::new();
        app.insert_resource(RuntimeFlags::from_args(
            args.iter().map(|arg| arg.to_string()),
        ));
        app.init_resource::<RuntimePackConfig>();
        app.init_resource::<LoadedPacks>();
        app.add_observer(
            |e: On<runtime_assets::pack::readers::LoadPackFileEvent>, mut loaded: ResMut<LoadedPacks>| {
                loaded.0.push(e.event().path.to_path_buf());
            },
        );
        app.add_plugins(ClientDevPlugin);
        app
    }

    #[test]
    fn dev_mode_allows_dir_read() {
        let app = app_with_pack_config(&["--dev"]);

        assert!(
            *app.world()
                .resource::<RuntimePackConfig>()
                .get_allow_dir_read()
        );
    }

    #[test]
    fn normal_mode_does_not_allow_dir_read() {
        let app = app_with_pack_config(&[]);

        assert!(
            !*app
                .world()
                .resource::<RuntimePackConfig>()
                .get_allow_dir_read()
        );
    }

    #[test]
    fn dev_mode_without_pack_config_does_not_panic() {
        assert!(dev_status(&["--dev", "--pack-file", "game.tbpk"]));
    }

    #[test]
    fn dev_mode_loads_pack_file() {
        let app = app_with_pack_config(&["--dev", "--pack-file", "game.tbpk"]);

        assert_eq!(
            app.world().resource::<LoadedPacks>().0,
            vec![PathBuf::from("game.tbpk")]
        );
    }

    #[test]
    fn dev_mode_loads_first_pack_file_only() {
        let app = app_with_pack_config(&["--dev", "--pack-file", "a.tbpk", "b.tbpk"]);

        assert_eq!(
            app.world().resource::<LoadedPacks>().0,
            vec![PathBuf::from("a.tbpk")]
        );
    }

    #[test]
    fn dev_mode_without_pack_file_loads_nothing() {
        let app = app_with_pack_config(&["--dev"]);

        assert!(app.world().resource::<LoadedPacks>().0.is_empty());
    }

    #[test]
    fn normal_mode_ignores_pack_file() {
        let app = app_with_pack_config(&["--pack-file", "game.tbpk"]);

        assert!(!*app.world().resource::<DeveloperModeStatus>().get_value());
        assert!(app.world().resource::<LoadedPacks>().0.is_empty());
    }

    #[test]
    #[should_panic(expected = "No pack file given in --pack-file")]
    fn pack_file_flag_without_value_panics() {
        dev_status(&["--dev", "--pack-file"]);
    }
}
