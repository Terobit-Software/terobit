//! `cargo xtask run client`: builds the client and runs it from a clean `out/client`, so
//! runtime output never mixes with `target/` and the working directory looks like an install.
//! `cargo xtask runbuild client` builds a pack's client scripts (and packs it with
//! `--package`) first, then does the same

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

use clap::Args;
use client_consts::flags::{DEV, PACK_FILE};
use client_consts::paths::{CLIENT_BIN, CLIENT_OUT};
use runtime_assets::ManifestSource;
use runtime_assets::pack::Target;
use runtime_consts::pack::ZSTD_LEVEL;
use runtime_consts::paths::TBPK_GENERATOR_OUT;
use snafu::{OptionExt, ResultExt, Snafu, ensure};
use tracing::{info, warn};

use crate::pkgen::{self, PkgenArgs, PkgenError, TargetArg, ZstdLevel};
use crate::scripts::{self, ScriptError};
use runtime_assets::ManifestLoadError;

/// pack loaded when `--pack-file` isn't given, relative to the workspace root
const DEFAULT_PACK_FILE: &str = "tests/examples/source_to_tbpk/basic1";

#[derive(Debug, Args)]
pub struct ClientArgs {
    /// Build with the `release` profile and without the `dev` feature, and run without the dev flags
    #[arg(long = "final")]
    r#final: bool,

    /// Run without the dev flags
    #[arg(long)]
    strip_defaults: bool,

    /// Pack or dev directory the client loads, relative to the current directory
    #[arg(
        long,
        default_value = DEFAULT_PACK_FILE,
        conflicts_with_all = ["final", "strip_defaults"],
    )]
    pack_file: PathBuf,
}

#[derive(Debug, Args)]
pub struct RunBuildClientArgs {
    /// The pack's manifest.toml, relative to the current directory
    #[arg(long)]
    manifest: PathBuf,

    /// Build the client pack with pkgen and run that, instead of the dev directory
    #[arg(long)]
    package: bool,

    /// Build with the `release` profile and without the `dev` feature, run without the dev
    /// flags, and pass `--final` to each script's build.sh
    #[arg(long = "final")]
    r#final: bool,

    /// Run without the dev flags
    #[arg(long)]
    strip_defaults: bool,
}

#[derive(Debug)]
enum BuildProfile {
    Release,
    ReleaseDev,
}

impl BuildProfile {
    /// name passed to `cargo build --profile`, also the directory under `target/` it builds to
    fn name(&self) -> &'static str {
        match self {
            BuildProfile::Release => "release",
            BuildProfile::ReleaseDev => "release-dev",
        }
    }
}

/// the client couldn't be built, staged or run
#[derive(Debug, Snafu)]
pub enum ClientError {
    #[snafu(display("failed to read the current directory, run this from the workspace root"))]
    CurrentDir { source: io::Error },

    #[snafu(display("failed to start `{command}`, is cargo on PATH?"))]
    StartBuild { command: String, source: io::Error },

    #[snafu(display(
        "`{command}` failed with {status}, not running the client, see the cargo output above"
    ))]
    BuildFailed { command: String, status: ExitStatus },

    #[snafu(display("failed to clean the old output folder `{}`", path.display()))]
    CleanOut { path: PathBuf, source: io::Error },

    #[snafu(display("failed to create the output folder `{}`", path.display()))]
    CreateOut { path: PathBuf, source: io::Error },

    #[snafu(display("failed to copy assets from `{}` to `{}`", from.display(), to.display()))]
    CopyAssets {
        from: PathBuf,
        to: PathBuf,
        source: io::Error,
    },

    #[snafu(display("failed to copy the built client from `{}` to `{}`", from.display(), to.display()))]
    CopyBin {
        from: PathBuf,
        to: PathBuf,
        source: io::Error,
    },

    #[snafu(display("failed to start the client `{}` with flags {flags:?}", bin.display()))]
    StartClient {
        bin: PathBuf,
        flags: Vec<String>,
        source: io::Error,
    },

    #[snafu(display("the client `{}` exited with {status}", bin.display()))]
    ClientFailed { bin: PathBuf, status: ExitStatus },

    #[snafu(display("failed to load manifest `{}`", path.display()))]
    LoadManifest {
        path: PathBuf,
        #[snafu(source(from(ManifestLoadError, Box::new)))]
        source: Box<ManifestLoadError>,
    },

    #[snafu(display("failed to build the client scripts"))]
    BuildScripts { source: ScriptError },

    #[snafu(display("failed to package `{}`", manifest.display()))]
    Package {
        manifest: PathBuf,
        #[snafu(source(from(PkgenError, Box::new)))]
        source: Box<PkgenError>,
    },

    #[snafu(display("pkgen succeeded but returned no client pack, this is a bug in xtask"))]
    NoClientPack,
}

/// `cargo xtask run client`
pub fn run(args: ClientArgs) -> Result<(), ClientError> {
    let cwd = std::env::current_dir().context(CurrentDirSnafu)?;
    // absolute, since the client runs with out_dir as its working directory
    let pack_file = cwd.join(&args.pack_file);
    launch(&cwd, args.r#final, args.strip_defaults, &pack_file)
}

/// `cargo xtask runbuild client`: builds the client scripts into the dev directory, or packs
/// it with `--package` (which builds them too), then runs the client on it
#[tracing::instrument(skip_all, fields(manifest = ?args.manifest, package = args.package, r#final = args.r#final))]
pub fn run_build(args: RunBuildClientArgs) -> Result<(), ClientError> {
    let cwd = std::env::current_dir().context(CurrentDirSnafu)?;

    let pack_file = if args.package {
        let pkgen_args = PkgenArgs {
            manifest: args.manifest.clone(),
            out: PathBuf::from(TBPK_GENERATOR_OUT),
            target: TargetArg::Client,
            zstd_level: ZstdLevel(Some(ZSTD_LEVEL)),
            r#final: args.r#final,
        };
        let packs = pkgen::run(&pkgen_args).context(PackageSnafu {
            manifest: &args.manifest,
        })?;
        packs.into_iter().next().context(NoClientPackSnafu)?
    } else {
        let source = ManifestSource::from_path(&args.manifest).context(LoadManifestSnafu {
            path: &args.manifest,
        })?;
        scripts::build_side(&source, Target::Client, args.r#final).context(BuildScriptsSnafu)?;
        source.get_root().to_path_buf()
    };
    if args.r#final || args.strip_defaults {
        // the client only loads a pack in dev mode, which these leave off
        warn!(
            "Running the client without --dev and --pack-file because of {}, it won't load {} or the scripts just built",
            if args.r#final {
                "--final"
            } else {
                "--strip-defaults"
            },
            pack_file.display()
        );
    } else {
        info!("Running the client on {}", pack_file.display());
    }

    // absolute, since the client runs with out_dir as its working directory
    launch(
        &cwd,
        args.r#final,
        args.strip_defaults,
        &cwd.join(pack_file),
    )
}

/// builds the client, stages it in a clean `out/client` and runs it there with the dev flags
/// pointing at `pack_file` (an absolute path), unless `is_final` or `strip_defaults`
#[tracing::instrument(skip_all, fields(is_final, strip_defaults, pack_file = ?pack_file))]
fn launch(
    cwd: &Path,
    is_final: bool,
    strip_defaults: bool,
    pack_file: &Path,
) -> Result<(), ClientError> {
    let assets_dir = cwd.join("bins").join(CLIENT_BIN).join("assets");
    let out_dir = cwd.join(CLIENT_OUT);
    info!("cwd: {cwd:?}, out dir: {out_dir:?}");

    let profile = if is_final {
        BuildProfile::Release
    } else {
        BuildProfile::ReleaseDev
    };
    info!("profile: {profile:?}");

    let pack_file = pack_file.to_string_lossy().into_owned();

    let mut flags: Vec<&str> = vec![];
    if !is_final && !strip_defaults {
        flags.extend([DEV, PACK_FILE, &pack_file]);
    }
    info!("client flags: {flags:?}");

    // Build the client, final builds never get developer mode
    let mut build_args = vec!["build", "--profile", profile.name(), "-p", CLIENT_BIN];
    if !is_final {
        build_args.extend(["--features", "dev"]);
    }
    let command = format!("cargo {}", build_args.join(" "));
    info!("Building client with `{command}`...");
    let build_status = Command::new("cargo")
        .args(&build_args)
        .status()
        .context(StartBuildSnafu { command: &command })?;
    ensure!(
        build_status.success(),
        BuildFailedSnafu {
            command,
            status: build_status
        }
    );

    // Recreate the output directory so stale files don't linger
    if out_dir.exists() {
        info!("Cleaning {out_dir:?}...");
        fs::remove_dir_all(&out_dir).context(CleanOutSnafu { path: &out_dir })?;
    }
    fs::create_dir_all(&out_dir).context(CreateOutSnafu { path: &out_dir })?;

    let out_assets = out_dir.join("assets");
    if assets_dir.exists() {
        info!("Copying assets...");
        copy_dir_all(&assets_dir, &out_assets).context(CopyAssetsSnafu {
            from: &assets_dir,
            to: &out_assets,
        })?;
    } else {
        warn!(
            "Assets folder {assets_dir:?} does not exist, the client will run without any of its bundled assets"
        );
    }

    let bin = format!("{CLIENT_BIN}{}", std::env::consts::EXE_SUFFIX);
    let built_bin = cwd.join("target").join(profile.name()).join(&bin);
    let out_bin = out_dir.join(&bin);
    info!("Copying {bin}...");
    fs::copy(&built_bin, &out_bin).context(CopyBinSnafu {
        from: &built_bin,
        to: &out_bin,
    })?;

    info!("Running {bin}...");
    let run_status = Command::new(&out_bin)
        .args(&flags)
        .current_dir(&out_dir)
        .status()
        .context(StartClientSnafu {
            bin: &out_bin,
            flags: flags.iter().map(|f| f.to_string()).collect::<Vec<_>>(),
        })?;
    ensure!(
        run_status.success(),
        ClientFailedSnafu {
            bin: &out_bin,
            status: run_status
        }
    );

    Ok(())
}

/// Recursively copy `src` into `dst`, creating `dst` if needed
fn copy_dir_all(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let dst_path = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_all(&entry.path(), &dst_path)?;
        } else {
            fs::copy(entry.path(), dst_path)?;
        }
    }
    Ok(())
}
