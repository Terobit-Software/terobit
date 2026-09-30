use bevy::{
    app::{PluginGroup, PluginGroupBuilder},
    picking::DefaultPickingPlugins,
};

pub struct RuntimePickingPlugins;

impl PluginGroup for RuntimePickingPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>().add_group(DefaultPickingPlugins)
    }
}
