use bevy::prelude::{App, AppExit, MessageWriter, MinimalPlugins, Res, Startup};
use runtime_assets::pack::writers::{PackWriter, PackWriterError};
use runtime_assets::pack::{Target, ZSTD_LEVEL};
use runtime_run_flags::{RuntimeFlags, RuntimeRunFlagsPlugin};
use snafu::{OptionExt, Report, ResultExt, Snafu, ensure};
use std::path::{Path, PathBuf};
use tracing::{error, info, warn};

/// folder the packs go in when `--out` isn't given
const DEFAULT_OUT: &str = "out/tbpk_generator";

/// run flags that don't make a valid set of `Args`
#[derive(Debug, Snafu)]
enum ArgsError {
    #[snafu(display("{flag} takes exactly one value, but was given {}: {values:?}", values.len()))]
    ValueCount { flag: String, values: Vec<String> },

    #[snafu(display(
        "--manifest <path to manifest.toml> is required, it is the dev directory's manifest and asset paths are relative to its folder"
    ))]
    MissingManifest,

    #[snafu(display("--target is `{value}`, expected client, server or both"))]
    UnknownTarget { value: String },

    #[snafu(display("--zstd-level is `{value}`, expected a whole number or `none`"))]
    InvalidZstdLevel {
        value: String,
        source: std::num::ParseIntError,
    },
}

/// pack generation stopped or one of the targets failed
#[derive(Debug, Snafu)]
enum GenerateError {
    #[snafu(display("failed to set up the pack writer"))]
    SetupWriter {
        #[snafu(source(from(PackWriterError, Box::new)))]
        source: Box<PackWriterError>,
    },

    #[snafu(display("failed to load manifest `{}`", path.display()))]
    LoadManifest {
        path: PathBuf,
        #[snafu(source(from(PackWriterError, Box::new)))]
        source: Box<PackWriterError>,
    },

    #[snafu(display("failed to create the output folder `{}`", path.display()))]
    CreateOutDir {
        path: PathBuf,
        source: std::io::Error,
    },

    #[snafu(display("failed to name the packs from the loaded manifest"))]
    NamePacks {
        #[snafu(source(from(PackWriterError, Box::new)))]
        source: Box<PackWriterError>,
    },

    #[snafu(display("failed to build the {target:?} pack"))]
    BuildPack {
        target: Target,
        #[snafu(source(from(PackWriterError, Box::new)))]
        source: Box<PackWriterError>,
    },

    #[snafu(display("failed to write the {target:?} pack to `{}`", path.display()))]
    WritePack {
        target: Target,
        path: PathBuf,
        #[snafu(source(from(PackWriterError, Box::new)))]
        source: Box<PackWriterError>,
    },

    #[snafu(display(
        "{} of {total} targets failed ({failed:?}), see the errors above for why",
        failed.len()
    ))]
    TargetsFailed { failed: Vec<Target>, total: usize },
}

/// read from the run flags:
/// `--manifest <path>` the dev directory's manifest.toml, asset paths are relative to its folder
/// `--out <dir>` folder the packs are written to, created if missing
/// `--target <client|server|both>` which packs to build, defaults to both
/// `--zstd-level <level|none>` zstd level chunks are compressed at, `none` stores them
/// uncompressed. defaults to `ZSTD_LEVEL`
struct Args {
    manifest: PathBuf,
    out: PathBuf,
    targets: &'static [Target],
    /// `None` is no compression
    zstd_level: Option<i32>,
}

impl Args {
    fn from_flags(flags: &RuntimeFlags) -> Result<Self, ArgsError> {
        let flags = flags.get_flags();
        let value = |flag: &str| -> Result<Option<&String>, ArgsError> {
            match flags.get(flag) {
                None => Ok(None),
                Some(values) => match values.as_slice() {
                    [value] => Ok(Some(value)),
                    _ => ValueCountSnafu {
                        flag,
                        values: values.clone(),
                    }
                    .fail(),
                },
            }
        };

        let manifest = value("--manifest")?.context(MissingManifestSnafu)?.into();
        let out = value("--out")?.map_or(DEFAULT_OUT, String::as_str).into();
        let targets: &[Target] = match value("--target")?.map(String::as_str) {
            None | Some("both") => &[Target::Client, Target::Server],
            Some("client") => &[Target::Client],
            Some("server") => &[Target::Server],
            Some(other) => return UnknownTargetSnafu { value: other }.fail(),
        };

        let zstd_level = match value("--zstd-level")? {
            None => Some(ZSTD_LEVEL),
            Some(level) if level == "none" => None,
            Some(level) => Some(
                level
                    .parse()
                    .context(InvalidZstdLevelSnafu { value: level })?,
            ),
        };

        Ok(Self {
            manifest,
            out,
            targets,
            zstd_level,
        })
    }
}

fn main() -> AppExit {
    let _guard = runtime_logging::init_logging("Tbpk", "Generator");

    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(RuntimeRunFlagsPlugin)
        .add_systems(Startup, generate)
        .run()
}

fn generate(flags: Res<RuntimeFlags>, mut exit: MessageWriter<AppExit>) {
    let args = match Args::from_flags(&flags) {
        Ok(args) => args,
        Err(e) => {
            error!("Invalid pack generator flags: {}", Report::from_error(&e));
            exit.write(AppExit::error());
            return;
        }
    };
    info!("Manifest: {}", args.manifest.display());
    info!("Output folder: {}", args.out.display());
    info!("Targets: {:?}", args.targets);
    info!("Zstd level: {:?}", args.zstd_level);

    let start = std::time::Instant::now();
    exit.write(match write_packs(&args) {
        Ok(()) => {
            info!("Generated all packs in {:?}", start.elapsed());
            AppExit::Success
        }
        Err(e) => {
            error!(
                "Pack generation failed after {:?}: {}",
                start.elapsed(),
                Report::from_error(&e)
            );
            AppExit::error()
        }
    });
}

/// a failed target is logged and doesn't stop the other, the error only says which failed
fn write_packs(args: &Args) -> Result<(), GenerateError> {
    // TODO: pass a PackSigner with `.with_signer()` once keys can be loaded,
    // until then packs are written unsigned
    warn!("No signing key loaded, writing unsigned packs");
    let mut writer = PackWriter::new()
        .with_zstd_level(args.zstd_level)
        .context(SetupWriterSnafu)?;
    writer
        .load_manifest(args.manifest.clone())
        .context(LoadManifestSnafu {
            path: &args.manifest,
        })?;

    std::fs::create_dir_all(&args.out).context(CreateOutDirSnafu { path: &args.out })?;
    info!("Output folder {} is ready", args.out.display());

    // `<realm_id>-<pack_version>.<target>.tbpk`, as suggested in the tbpk doc
    let source = writer.get_manifest_source().context(NamePacksSnafu)?;
    let stem = format!("{}-{}", source.get_realm_id(), source.get_pack_version());

    info!("Packs will be named {stem}.<target>.tbpk");

    let mut failed = Vec::new();
    for &target in args.targets {
        let target_start = std::time::Instant::now();
        info!("Generating {target:?} pack");
        let path = args.out.join(format!("{stem}.{}.tbpk", side_name(target)));
        if let Err(e) = write_pack(&mut writer, target, &path) {
            error!(
                "Skipping the {target:?} pack after {:?}: {}",
                target_start.elapsed(),
                Report::from_error(&e)
            );
            failed.push(target);
            continue;
        }
        info!(
            "Generated {target:?} pack at {} in {:?}",
            path.display(),
            target_start.elapsed()
        );
    }

    ensure!(
        failed.is_empty(),
        TargetsFailedSnafu {
            failed,
            total: args.targets.len()
        }
    );
    Ok(())
}

fn write_pack(writer: &mut PackWriter, target: Target, path: &Path) -> Result<(), GenerateError> {
    writer
        .build_tbpk(target)
        .context(BuildPackSnafu { target })?;
    writer
        .write_tbpk_out(path)
        .context(WritePackSnafu { target, path })
}

fn side_name(target: Target) -> &'static str {
    match target {
        Target::Client => "client",
        Target::Server => "server",
    }
}
