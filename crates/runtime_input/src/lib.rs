use bevy::{
    app::{PluginGroup, PluginGroupBuilder},
    gilrs::GilrsPlugin,
    input::InputPlugin,
    input_focus::{InputDispatchPlugin, InputFocusPlugin},
};

pub struct TerobitInputPlugins;

impl PluginGroup for TerobitInputPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(InputPlugin)
            .add(InputFocusPlugin)
            .add(InputDispatchPlugin)
            .add(GilrsPlugin)
    }
}
