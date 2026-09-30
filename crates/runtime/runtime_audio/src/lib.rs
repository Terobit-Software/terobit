use bevy::{
    app::{PluginGroup, PluginGroupBuilder},
    audio::AudioPlugin,
};

pub struct RuntimeAudioPlugins;

impl PluginGroup for RuntimeAudioPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>().add(AudioPlugin::default())
    }
}
