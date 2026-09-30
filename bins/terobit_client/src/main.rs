use bevy::prelude::*;
use runtime_animation::TerobitAnimationPlugins;
use runtime_assets::TerobitAssetPlugins;
use runtime_audio::TerobitAudioPlugins;
use runtime_core::TerobitCorePlugins;
use runtime_input::TerobitInputPlugins;
use runtime_picking::TerobitPickingPlugins;
use runtime_render::TerobitRenderPlugins;
use runtime_scene::TerobitScenePlugins;
use runtime_ui::TerobitUiPlugins;
use runtime_window::TerobitWindowPlugins;

fn main() {
    let _guard = runtime_logging::init_logging();

    // order matters: assets before window/render, render before ui
    App::new()
        .add_plugins(TerobitCorePlugins)
        .add_plugins(TerobitInputPlugins)
        .add_plugins(TerobitAssetPlugins)
        .add_plugins(TerobitScenePlugins)
        .add_plugins(TerobitWindowPlugins)
        .add_plugins(TerobitRenderPlugins)
        .add_plugins(TerobitUiPlugins)
        .add_plugins(TerobitAnimationPlugins)
        .add_plugins(TerobitAudioPlugins)
        .add_plugins(TerobitPickingPlugins)
        .run();
}
