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
use runtime_assets::RuntimePackReaderPlugin;
use runtime_run_flags::RuntimeRunFlagsPlugin;
use runtime_sandbox::RuntimeSandboxPlugin;

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
            // registers the `pack://` asset source, which has to happen before `AssetPlugin`
            .add(RuntimePackReaderPlugin)
            // scene, window and render all need the `AssetServer`
            .add(AssetPlugin::default())
            .add(RuntimeRunFlagsPlugin)
            .add(RuntimeSandboxPlugin)
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
                .get_resource::<runtime_assets::RuntimePackConfig>()
                .is_some()
        );
    }
}
