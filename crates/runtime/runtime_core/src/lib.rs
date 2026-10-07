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
use runtime_assets::pack::readers::RuntimePackReaderPlugin;
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

#[cfg(test)]
mod plugin_tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn adds_runtime_resources() {
        let mut app = App::new();
        app.add_plugins(RuntimeCorePlugins);

        assert!(
            app.world()
                .get_resource::<runtime_run_flags::RuntimeFlags>()
                .is_some()
        );
        assert!(
            app.world()
                .get_resource::<runtime_assets::pack::readers::RuntimePackConfig>()
                .is_some()
        );
    }
}
