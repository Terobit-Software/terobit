use bevy::prelude::*;
use client_dev::ClientDevPlugin;
use runtime_core::RuntimeCorePlugins;
use runtime_pack::writers::RuntimePackWriterPlugin;

fn main() {
    let _guard = runtime_logging::init_logging("Tbpk", "Generator");

    App::new()
        .add_plugins(RuntimeCorePlugins)
        .add_plugins(ClientDevPlugin)
        .add_plugins(RuntimePackWriterPlugin)
        .run();
}
