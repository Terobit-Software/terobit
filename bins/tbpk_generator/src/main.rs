use std::path::PathBuf;

use bevy::prelude::*;
use client_dev::ClientDevPlugin;
use runtime_assets::pack::Target;
use runtime_assets::pack::writers::PackWriter;
use runtime_core::RuntimeCorePlugins;
use std::path::Path;

fn main() {
    let _guard = runtime_logging::init_logging("Tbpk", "Generator");

    App::new()
        .add_plugins(RuntimeCorePlugins)
        .add_plugins(ClientDevPlugin)
        .add_systems(Update, || {
            // TODO: pass a PackSigner with `.with_signer()` once keys can be loaded,
            // until then packs are written unsigned
            let mut writer = PackWriter::new();
            for target in [Target::Client, Target::Server] {
                if let Err(msg) = writer.build_tbpk(target) {
                    tracing::error!("error on build tbpk for {:?}: {}", target, msg);
                    continue;
                }
                if let Err(msg) = writer.write_tbpk_out() {
                    tracing::error!("error on write tbpk out for {:?}: {}", target, msg);
                };
            }
        })
        .run();
}
