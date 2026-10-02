use bevy::prelude::*;

pub struct ClientDevPlugin;

impl Plugin for ClientDevPlugin {
    fn build(&self, app: &mut App) {
        if app
            .world()
            .resource::<runtime_run_flags::RuntimeFlags>()
            .flags
            .contains_key("--dev")
        {
            tracing::info!(
                "\n\n--------------------------------\nClient running in DEVELOPER mode\n--------------------------------\n"
            );
        } else {
            tracing::info!("Client running in normal mode");
        }
    }
}
