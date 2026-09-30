use bevy::{
    app::{PluginGroup, PluginGroupBuilder},
    text::TextPlugin,
    ui::UiPlugin,
    ui_render::UiRenderPlugin,
    ui_widgets::UiWidgetsPlugins,
};

/// Must be added after `RuntimeRenderPlugins`.
pub struct RuntimeUiPlugins;

impl PluginGroup for RuntimeUiPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(TextPlugin)
            .add(UiPlugin)
            .add(UiRenderPlugin)
            .add_group(UiWidgetsPlugins)
    }
}
