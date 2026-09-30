use bevy::app::PluginGroup;
use bevy::app::PluginGroupBuilder;
use bevy::asset::AssetPlugin;

pub struct TerobitAssetPlugins;

impl PluginGroup for TerobitAssetPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>().add(AssetPlugin::default())
    }
}
