use bevy::{
    app::{
        PanicHandlerPlugin, PluginGroup, PluginGroupBuilder, TaskPoolPlugin,
        TerminalCtrlCHandlerPlugin,
    },
    diagnostic::{DiagnosticsPlugin, FrameCountPlugin},
    state::app::StatesPlugin,
    time::TimePlugin,
    transform::TransformPlugin,
};

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
    }
}
