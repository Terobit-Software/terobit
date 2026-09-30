use bevy::app::PluginGroup;
use bevy::app::PluginGroupBuilder;
use bevy::asset::AssetPlugin;

pub struct RuntimeAssetPlugins;

impl PluginGroup for RuntimeAssetPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>().add(AssetPlugin::default())
    }
}
