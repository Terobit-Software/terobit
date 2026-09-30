use bevy::{
    animation::AnimationPlugin,
    app::{PluginGroup, PluginGroupBuilder},
};

pub struct RuntimeAnimationPlugins;

impl PluginGroup for RuntimeAnimationPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>().add(AnimationPlugin)
    }
}
