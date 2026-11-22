use bevy::{
    a11y::AccessibilityPlugin,
    app::{PluginGroup, PluginGroupBuilder},
    clipboard::ClipboardPlugin,
    window::WindowPlugin,
    winit::WinitPlugin,
};

/// Must be added after `RuntimeCorePlugins` (custom cursors need the `AssetServer`).
pub struct RuntimeWindowPlugins;

impl PluginGroup for RuntimeWindowPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(WindowPlugin::default())
            .add(AccessibilityPlugin)
            .add(WinitPlugin::default())
            .add(ClipboardPlugin)
    }
}
