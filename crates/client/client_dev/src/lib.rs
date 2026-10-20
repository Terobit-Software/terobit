use bevy::prelude::*;
use runtime_assets::RuntimePackConfig;
use std::path::PathBuf;
use tracing::{error, warn};

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

            let pack_file = flags.get("--pack-file").and_then(|values| {
                match values.as_slice() {
                    [] => {
                        error!(
                            "--pack-file was given without a path, expected `--pack-file <path to .tbpk or dev directory>`. No pack will be loaded"
                        );
                        None
                    }
                    [first, rest @ ..] => {
                        if !rest.is_empty() {
                            warn!(
                                "--pack-file was given {} paths, only the first (`{first}`) is loaded, ignoring {rest:?}",
                                values.len()
                            );
                        }
                        Some(PathBuf::from(first).into_boxed_path())
                    }
                }
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
                    "Developer mode couldn't allow reading dev directories as packs, the RuntimePackConfig resource doesn't exist. Is RuntimePackReaderPlugin added before ClientDevPlugin?"
                )
            }

            if let Some(path) = pack_file {
                app.world_mut()
                    .trigger(runtime_assets::LoadPackFileEvent { path });
            } else {
                warn!(
                    "Developer mode is on but no pack will be loaded, pass `--pack-file <path to .tbpk or dev directory>` to load one"
                )
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
            |e: On<runtime_assets::LoadPackFileEvent>, mut loaded: ResMut<LoadedPacks>| {
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
    fn pack_file_flag_without_value_loads_nothing() {
        let app = app_with_pack_config(&["--dev", "--pack-file"]);

        assert!(*app.world().resource::<DeveloperModeStatus>().get_value());
        assert!(app.world().resource::<LoadedPacks>().0.is_empty());
    }
}
