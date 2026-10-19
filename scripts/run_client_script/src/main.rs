use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, ExitStatus};

use clap::Parser;
use snafu::{Report, ResultExt, Snafu, ensure};
use tracing::{error, info, warn};

/// Build and run the terobit client
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Build in final (release) mode
    #[arg(short, long)]
    r#final: bool,
    /// Don't pass the default dev flags to the client
    #[arg(short, long)]
    strip_defaults: bool,
}

#[allow(dead_code)]
#[derive(Debug)]
enum BuildProfile {
    Release,
    Dev,
    ReleaseDev,
}

impl BuildProfile {
    /// Name passed to `cargo build --profile`
    fn name(&self) -> &'static str {
        match self {
            BuildProfile::Release => "release",
            BuildProfile::Dev => "dev",
            BuildProfile::ReleaseDev => "release-dev",
        }
    }

    /// Directory under `target/` that cargo writes this profile to
    fn target_dir(&self) -> &'static str {
        match self {
            BuildProfile::Dev => "debug",
            other => other.name(),
        }
    }
}

const BIN_NAME: &str = "terobit_client";

/// the client couldn't be built, staged or run
#[derive(Debug, Snafu)]
enum RunError {
    #[snafu(display("failed to read the current directory, run this from the workspace root"))]
    CurrentDir { source: io::Error },

    #[snafu(display(
        "failed to start `cargo build --profile {profile} -p {BIN_NAME}`, is cargo on PATH?"
    ))]
    StartBuild {
        profile: &'static str,
        source: io::Error,
    },

    #[snafu(display(
        "`cargo build --profile {profile} -p {BIN_NAME}` failed with {status}, not running the client, see the cargo output above"
    ))]
    BuildFailed {
        profile: &'static str,
        status: ExitStatus,
    },

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
}

fn main() -> ExitCode {
    // returning ExitCode (instead of process::exit) lets the guard drop and flush logs
    let _guard = runtime_logging::init_logging("Tbrun", "Client");

    match run(Args::parse()) {
        Ok(()) => {
            info!("Run succeeded");
            ExitCode::SUCCESS
        }
        Err(e) => {
            error!("Run client failed: {}", Report::from_error(&e));
            ExitCode::FAILURE
        }
    }
}

fn run(args: Args) -> Result<(), RunError> {
    let cwd = std::env::current_dir().context(CurrentDirSnafu)?;
    info!("cwd: {:?}", cwd);

    let client_dir = cwd.join("bins/terobit_client");
    info!("client dir: {:?}", client_dir.canonicalize());

    let assets_dir = client_dir.join("assets");
    info!("assets dir: {:?}", assets_dir.canonicalize());

    let out_dir = cwd.join("out/client");
    info!("out dir: {:?}", out_dir.canonicalize());

    let is_final = args.r#final;
    info!("final: {is_final}");

    let profile = if is_final {
        BuildProfile::Release
    } else {
        BuildProfile::ReleaseDev
    };
    info!("profile: {:?}", profile);

    // absolute, since the client runs with out_dir as its working directory
    let pack_file = cwd
        .join("tests")
        .join("examples")
        .join("source_to_tbpk")
        .join("basic1")
        .to_string_lossy()
        .into_owned();

    let mut flags: Vec<&str> = vec![];
    if !is_final && !args.strip_defaults {
        flags.extend(["--dev", "--pack-file", &pack_file]);
    }
    info!("client flags: {:?}", flags);

    // Build the client
    info!("Building client with profile {}...", profile.name());
    let build_status = Command::new("cargo")
        .args(["build", "--profile", profile.name(), "-p", BIN_NAME])
        .status()
        .context(StartBuildSnafu {
            profile: profile.name(),
        })?;
    ensure!(
        build_status.success(),
        BuildFailedSnafu {
            profile: profile.name(),
            status: build_status
        }
    );
    info!("Build succeeded");

    // Recreate the output directory so stale files don't linger
    info!("Copying assets...");
    if out_dir.exists() {
        info!("Folder {:?} already exists, cleaning...", out_dir);
        fs::remove_dir_all(&out_dir).context(CleanOutSnafu { path: &out_dir })?;
    } else {
        info!("Folder {:?} does not exist, creating...", out_dir);
    }
    fs::create_dir_all(&out_dir).context(CreateOutSnafu { path: &out_dir })?;

    // Copy the assets to the output directory
    let out_assets = out_dir.join("assets");
    if !assets_dir.exists() {
        warn!(
            "Assets folder {:?} does not exist, the client will run without any of its bundled assets",
            assets_dir
        );
    } else {
        copy_dir_all(&assets_dir, &out_assets).context(CopyAssetsSnafu {
            from: &assets_dir,
            to: &out_assets,
        })?;
    }

    // Copy the binary to the output directory
    let bin = format!("{BIN_NAME}{}", std::env::consts::EXE_SUFFIX);
    info!("Copying {bin}...");
    let built_bin = cwd.join("target").join(profile.target_dir()).join(&bin);
    let out_bin = out_dir.join(&bin);
    fs::copy(&built_bin, &out_bin).context(CopyBinSnafu {
        from: &built_bin,
        to: &out_bin,
    })?;

    // Start run
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
