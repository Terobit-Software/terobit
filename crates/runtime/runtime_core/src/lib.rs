use bevy::{
    app::{
        PanicHandlerPlugin, PluginGroup, PluginGroupBuilder, TaskPoolPlugin,
        TerminalCtrlCHandlerPlugin,
    },
    asset::AssetPlugin,
    diagnostic::{DiagnosticsPlugin, FrameCountPlugin},
    state::app::StatesPlugin,
    time::TimePlugin,
    transform::TransformPlugin,
};
use runtime_pack::readers::RuntimePackReaderPlugin;
use runtime_run_flags::RuntimeRunFlagsPlugin;

pub struct RuntimeCorePlugins;

impl PluginGroup for RuntimeCorePlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(PanicHandlerPlugin)
            .add(TaskPoolPlugin::default())
            .add(FrameCountPlugin)
            .add(TimePlugin)
            .add(TransformPlugin)
            .add(DiagnosticsPlugin)
            .add(TerminalCtrlCHandlerPlugin)
            .add(StatesPlugin)
            // scene, window and render all need the `AssetServer`
            .add(AssetPlugin::default())
            .add(RuntimeRunFlagsPlugin)
            .add(RuntimePackReaderPlugin)
    }
}
