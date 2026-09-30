use bevy::{
    app::{PluginGroup, PluginGroupBuilder},
    scene::ScenePlugin,
    world_serialization::WorldSerializationPlugin,
};

pub struct RuntimeScenePlugins;

impl PluginGroup for RuntimeScenePlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(WorldSerializationPlugin)
            .add(ScenePlugin)
    }
}
