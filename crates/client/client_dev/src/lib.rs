use bevy::prelude::*;
use runtime_pack::RuntimePackConfig;
use std::path::PathBuf;

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
            }

            if let Some(path) = pack_file {
                app.world_mut()
                    .trigger(runtime_pack::LoadPackFileEvent { path });
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
}
