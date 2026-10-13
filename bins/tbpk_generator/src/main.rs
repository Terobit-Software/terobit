use std::path::PathBuf;

use bevy::prelude::*;
use client_dev::ClientDevPlugin;
use runtime_core::RuntimeCorePlugins;
use std::path::Path;

fn main() {
    let _guard = runtime_logging::init_logging("Tbpk", "Generator");

    App::new()
        .add_plugins(RuntimeCorePlugins)
        .add_plugins(ClientDevPlugin)
        .add_systems(Update, || {
            let mut writer = runtime_assets::pack::writers::PackWriter::new();
            if let Err(msg) = writer.write_tbpk_out() {
                tracing::error!("error on write tbpk out: {}", msg);
            };
        })
        .run();
}
