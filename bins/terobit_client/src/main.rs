use bevy::prelude::*;
use client_dev::ClientDevPlugin;
use runtime_animation::RuntimeAnimationPlugins;
use runtime_assets::RuntimeAssetPlugins;
use runtime_audio::RuntimeAudioPlugins;
use runtime_core::RuntimeCorePlugins;
use runtime_input::RuntimeInputPlugins;
use runtime_picking::RuntimePickingPlugins;
use runtime_render::RuntimeRenderPlugins;
use runtime_scene::RuntimeScenePlugins;
use runtime_ui::RuntimeUiPlugins;
use runtime_window::RuntimeWindowPlugins;

fn main() {
    let _guard = runtime_logging::init_logging();

    // order matters: assets before window/render, render before ui
    App::new()
        .add_plugins(RuntimeCorePlugins)
        .add_plugins(RuntimeInputPlugins)
        .add_plugins(RuntimeAssetPlugins)
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
