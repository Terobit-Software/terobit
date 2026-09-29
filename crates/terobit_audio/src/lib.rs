use bevy::{
    app::{PluginGroup, PluginGroupBuilder},
    audio::AudioPlugin,
};

pub struct TerobitAudioPlugins;

impl PluginGroup for TerobitAudioPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>().add(AudioPlugin::default())
    }
}
