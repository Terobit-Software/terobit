use bevy::{
    app::{PluginGroup, PluginGroupBuilder},
    text::TextPlugin,
    ui::UiPlugin,
    ui_render::UiRenderPlugin,
    ui_widgets::UiWidgetsPlugins,
};

/// Must be added after `TerobitRenderPlugins`.
pub struct TerobitUiPlugins;

impl PluginGroup for TerobitUiPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(TextPlugin)
            .add(UiPlugin)
            .add(UiRenderPlugin)
            .add_group(UiWidgetsPlugins)
    }
}
