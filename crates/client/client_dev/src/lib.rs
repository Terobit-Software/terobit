use bevy::prelude::*;

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
            .resource::<runtime_run_flags::RuntimeFlags>()
            .get_flags()
            .contains_key("--dev")
        {
            tracing::info!(
                "\n\n--------------------------------\nClient running in DEVELOPER mode\n--------------------------------\n"
            );
            app.world_mut()
                .insert_resource(DeveloperModeStatus { value: true });
        } else {
            tracing::info!("Client running in normal mode");
            app.world_mut()
                .insert_resource(DeveloperModeStatus { value: false });
        }
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
