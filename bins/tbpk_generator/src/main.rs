use bevy::prelude::*;
use client_dev::ClientDevPlugin;
use runtime_assets::pack::Target;
use runtime_assets::pack::writers::PackWriter;
use runtime_core::RuntimeCorePlugins;
use std::path::Path;

/// the example manifest the generator packs, relative to this crate
const EXAMPLE_MANIFEST: &str = "../../tests/examples/source_to_tbpk/basic1/manifest.toml";

fn main() -> AppExit {
    let _guard = runtime_logging::init_logging("Tbpk", "Generator");

    App::new()
        .add_plugins(RuntimeCorePlugins)
        .add_plugins(ClientDevPlugin)
        .add_systems(Startup, |mut exit: MessageWriter<AppExit>| {
            // TODO: pass a PackSigner with `.with_signer()` once keys can be loaded,
            // until then packs are written unsigned
            let mut writer = PackWriter::new();
            let manifest_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(EXAMPLE_MANIFEST);
            if let Err(msg) = writer.load_manifest(manifest_path.clone()) {
                tracing::error!(
                    "error on load manifest {}: {}",
                    manifest_path.display(),
                    msg
                );
                exit.write(AppExit::error());
                return;
            }

            let mut failed = false;
            for target in [Target::Client, Target::Server] {
                if let Err(msg) = writer.build_tbpk(target) {
                    tracing::error!("error on build tbpk for {:?}: {}", target, msg);
                    failed = true;
                    continue;
                }
                if let Err(msg) = writer.write_tbpk_out() {
                    tracing::error!("error on write tbpk out for {:?}: {}", target, msg);
                    failed = true;
                };
            }

            exit.write(if failed {
                AppExit::error()
            } else {
                AppExit::Success
            });
        })
        .run()
}
