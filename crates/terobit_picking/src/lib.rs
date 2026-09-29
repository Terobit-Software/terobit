use bevy::{
    app::{PluginGroup, PluginGroupBuilder},
    picking::DefaultPickingPlugins,
};

pub struct TerobitPickingPlugins;

impl PluginGroup for TerobitPickingPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>().add_group(DefaultPickingPlugins)
    }
}
