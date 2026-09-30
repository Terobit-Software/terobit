use bevy::{
    app::{PluginGroup, PluginGroupBuilder},
    scene::ScenePlugin,
    world_serialization::WorldSerializationPlugin,
};

pub struct TerobitScenePlugins;

impl PluginGroup for TerobitScenePlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(WorldSerializationPlugin)
            .add(ScenePlugin)
    }
}
