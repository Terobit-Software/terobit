use bevy::{
    animation::AnimationPlugin,
    app::{PluginGroup, PluginGroupBuilder},
};

pub struct TerobitAnimationPlugins;

impl PluginGroup for TerobitAnimationPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>().add(AnimationPlugin)
    }
}
