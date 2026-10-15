use bevy::prelude::*;
use client_dev::ClientDevPlugin;
use runtime_assets::pack::Target;
use runtime_assets::pack::writers::PackWriter;
use runtime_core::RuntimeCorePlugins;
use runtime_run_flags::RuntimeFlags;
use std::path::PathBuf;

/// folder the packs go in when `--out` isn't given
const DEFAULT_OUT: &str = "out/tbpk_generator";

/// read from the run flags:
/// `--manifest <path>` the dev directory's manifest.toml, asset paths are relative to its folder
/// `--out <dir>` folder the packs are written to, created if missing
/// `--target <client|server|both>` which packs to build, defaults to both
struct Args {
    manifest: PathBuf,
    out: PathBuf,
    targets: &'static [Target],
}

impl Args {
    fn from_flags(flags: &RuntimeFlags) -> Result<Self, String> {
        let flags = flags.get_flags();
        let value = |flag: &str| -> Result<Option<&String>, String> {
            match flags.get(flag) {
                None => Ok(None),
                Some(values) => match values.as_slice() {
                    [value] => Ok(Some(value)),
                    _ => Err(format!("{flag} takes exactly one value")),
                },
            }
        };

        let manifest = value("--manifest")?
            .ok_or("--manifest <path to manifest.toml> is required")?
            .into();
        let out = value("--out")?.map_or(DEFAULT_OUT, String::as_str).into();
        let targets: &[Target] = match value("--target")?.map(String::as_str) {
            None | Some("both") => &[Target::Client, Target::Server],
            Some("client") => &[Target::Client],
            Some("server") => &[Target::Server],
            Some(other) => {
                return Err(format!(
                    "--target is `{other}`, expected client, server or both"
                ));
            }
        };

        Ok(Self {
            manifest,
            out,
            targets,
        })
    }
}

fn main() -> AppExit {
    let _guard = runtime_logging::init_logging("Tbpk", "Generator");

    App::new()
        .add_plugins(RuntimeCorePlugins)
        .add_plugins(ClientDevPlugin)
        .add_systems(Startup, generate)
        .run()
}

fn generate(flags: Res<RuntimeFlags>, mut exit: MessageWriter<AppExit>) {
    let args = match Args::from_flags(&flags) {
        Ok(args) => args,
        Err(msg) => {
            tracing::error!("{msg}");
            exit.write(AppExit::error());
            return;
        }
    };

    exit.write(match write_packs(&args) {
        Ok(()) => AppExit::Success,
        Err(()) => AppExit::error(),
    });
}

/// errors are logged as they happen, so one failed target doesn't stop the other
fn write_packs(args: &Args) -> Result<(), ()> {
    // TODO: pass a PackSigner with `.with_signer()` once keys can be loaded,
    // until then packs are written unsigned
    let mut writer = PackWriter::new();
    if let Err(msg) = writer.load_manifest(args.manifest.clone()) {
        tracing::error!(
            "error on load manifest {}: {}",
            args.manifest.display(),
            msg
        );
        return Err(());
    }

    if let Err(e) = std::fs::create_dir_all(&args.out) {
        tracing::error!("error on create {}: {}", args.out.display(), e);
        return Err(());
    }

    // `<realm_id>-<pack_version>.<target>.tbpk`, as suggested in the tbpk doc
    let stem = match writer.get_manifest_source() {
        Ok(source) => format!("{}-{}", source.get_realm_id(), source.get_pack_version()),
        Err(msg) => {
            tracing::error!("{msg}");
            return Err(());
        }
    };

    let mut failed = false;
    for &target in args.targets {
        if let Err(msg) = writer.build_tbpk(target) {
            tracing::error!("error on build tbpk for {:?}: {}", target, msg);
            failed = true;
            continue;
        }

        let side = match target {
            Target::Client => "client",
            Target::Server => "server",
        };
        let path = args.out.join(format!("{stem}.{side}.tbpk"));
        if let Err(msg) = writer.write_tbpk_out(&path) {
            tracing::error!("error on write tbpk out for {:?}: {}", target, msg);
            failed = true;
        }
    }

    if failed { Err(()) } else { Ok(()) }
}
