use bevy::prelude::*;
use client_dev::ClientDevPlugin;
use runtime_core::{
    RuntimeAnimationPlugins, RuntimeAudioPlugins, RuntimeCorePlugins, RuntimeInputPlugins,
    RuntimePickingPlugins, RuntimeRenderPlugins, RuntimeScenePlugins, RuntimeUiPlugins,
    RuntimeWindowPlugins,
};

fn main() {
    // the client logging name
    let (first, last) = client_consts::logging::CLIENT;
    // keeps the file logging guard alive
    let _guard = runtime_logging::init_logging(first, last);

    // order does matter, make sure runtime plugins are loaded before client plugins
    // unless explicitly required
    App::new()
        .add_plugins(RuntimeCorePlugins)
        .add_plugins(RuntimeInputPlugins)
        .add_plugins(RuntimeScenePlugins)
        .add_plugins(RuntimeWindowPlugins)
        .add_plugins(RuntimeRenderPlugins)
        .add_plugins(RuntimeUiPlugins)
        .add_plugins(RuntimeAnimationPlugins)
        .add_plugins(RuntimeAudioPlugins)
        .add_plugins(RuntimePickingPlugins)
        // add client plugins AFTER runtime
        .add_plugins(ClientDevPlugin)
        .run();
}
