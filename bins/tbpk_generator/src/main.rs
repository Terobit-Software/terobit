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
            let _manifest = writer.read_manifest(
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../tests/examples/source_to_tbpk/basic1/manifest.toml"),
            );
            if let Err(msg) = writer.prep_assets() {
                tracing::error!("error on prep assets: {}", msg);
            };
            if let Err(msg) = writer.load_then_write() {
                tracing::error!("error on load then write: {}", msg);
            };
        })
        .run();
}
