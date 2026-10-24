use bevy::prelude::*;
use runtime_run_flags::RuntimeFlags;

#[cfg(feature = "dev")]
mod dev;

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
        if app
            .world()
            .resource::<RuntimeFlags>()
            .get_flags()
            .contains_key("--dev")
        {
            #[cfg(feature = "dev")]
            {
                dev::enable(app);
                return;
            }
            #[cfg(not(feature = "dev"))]
            tracing::warn!(
                "Ignoring --dev, this client was built without the `dev` feature. Build it with `--features dev` (`cargo rcl` does) to use developer mode"
            );
        }

        tracing::info!("Client running in normal mode");
        app.world_mut()
            .insert_resource(DeveloperModeStatus { value: false });
    }
}

#[cfg(test)]
mod plugin_tests {
    use super::*;
    use runtime_assets::{LoadPackFileEvent, RuntimePackConfig};
    use std::path::PathBuf;

    fn dev_status(args: &[&str]) -> bool {
        let mut app = App::new();
        app.insert_resource(RuntimeFlags::from_args(
            args.iter().map(|arg| arg.to_string()),
        ));
        app.add_plugins(ClientDevPlugin);

        *app.world().resource::<DeveloperModeStatus>().get_value()
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
            |e: On<LoadPackFileEvent>, mut loaded: ResMut<LoadedPacks>| {
                loaded.0.push(e.event().path.to_path_buf());
            },
        );
        app.add_plugins(ClientDevPlugin);
        app
    }

    fn allows_dir_read(app: &App) -> bool {
        *app.world()
            .resource::<RuntimePackConfig>()
            .get_allow_dir_read()
    }

    #[test]
    fn sets_dev_status_false() {
        assert!(!dev_status(&[]));
    }

    #[test]
    fn normal_mode_does_not_allow_dir_read() {
        assert!(!allows_dir_read(&app_with_pack_config(&[])));
    }

    #[test]
    fn normal_mode_ignores_pack_file() {
        let app = app_with_pack_config(&["--pack-file", "game.tbpk"]);

        assert!(!*app.world().resource::<DeveloperModeStatus>().get_value());
        assert!(app.world().resource::<LoadedPacks>().0.is_empty());
    }

    #[cfg(not(feature = "dev"))]
    #[test]
    fn dev_flag_is_ignored_without_dev_feature() {
        let app = app_with_pack_config(&["--dev", "--pack-file", "game.tbpk"]);

        assert!(!*app.world().resource::<DeveloperModeStatus>().get_value());
        assert!(!allows_dir_read(&app));
        assert!(app.world().resource::<LoadedPacks>().0.is_empty());
    }

    #[cfg(feature = "dev")]
    mod dev {
        use super::*;

        #[test]
        fn sets_dev_status_true() {
            assert!(dev_status(&["--dev"]));
        }

        #[test]
        fn dev_mode_allows_dir_read() {
            assert!(allows_dir_read(&app_with_pack_config(&["--dev"])));
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
        fn pack_file_flag_without_value_loads_nothing() {
            let app = app_with_pack_config(&["--dev", "--pack-file"]);

            assert!(*app.world().resource::<DeveloperModeStatus>().get_value());
            assert!(app.world().resource::<LoadedPacks>().0.is_empty());
        }
    }
}
