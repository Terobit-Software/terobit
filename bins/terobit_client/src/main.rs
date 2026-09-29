use bevy::prelude::*;
use terobit_animation::TerobitAnimationPlugins;
use terobit_assets::TerobitAssetPlugins;
use terobit_audio::TerobitAudioPlugins;
use terobit_core::TerobitCorePlugins;
use terobit_input::TerobitInputPlugins;
use terobit_picking::TerobitPickingPlugins;
use terobit_render::TerobitRenderPlugins;
use terobit_scene::TerobitScenePlugins;
use terobit_ui::TerobitUiPlugins;
use terobit_window::TerobitWindowPlugins;

fn main() {
    let _guard = terobit_logging::init_logging();

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
