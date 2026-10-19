use std::fs;
use std::io;
use std::path::Path;
use std::process::{Command, ExitCode};

use clap::Parser;
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

fn main() -> ExitCode {
    // returning ExitCode (instead of process::exit) lets the guard drop and flush logs
    let _guard = runtime_logging::init_logging("Tbrun", "Client");

    let args = Args::parse();

    let cwd = std::env::current_dir().unwrap();
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
    let build_status = match Command::new("cargo")
        .args(["build", "--profile", profile.name(), "-p", BIN_NAME])
        .status()
    {
        Ok(status) => status,
        Err(e) => {
            error!("Failed to start cargo: {e}");
            return ExitCode::FAILURE;
        }
    };
    if !build_status.success() {
        error!("Build failed with {build_status} - not running client");
        return ExitCode::FAILURE;
    }
    info!("Build succeeded");

    // Recreate the output directory so stale files don't linger
    info!("Copying assets...");
    if out_dir.exists() {
        info!("Folder {:?} already exists, cleaning...", out_dir);
        if let Err(e) = fs::remove_dir_all(&out_dir) {
            error!("Failed to clean {:?}: {e}", out_dir);
            return ExitCode::FAILURE;
        }
    } else {
        info!("Folder {:?} does not exist, creating...", out_dir);
    }
    if let Err(e) = fs::create_dir_all(&out_dir) {
        error!("Failed to create {:?}: {e}", out_dir);
        return ExitCode::FAILURE;
    }

    // Copy the assets to the output directory
    if !assets_dir.exists() {
        warn!("Assets folder {:?} does not exist, skipping...", assets_dir);
    } else if let Err(e) = copy_dir_all(&assets_dir, &out_dir.join("assets")) {
        error!("Failed to copy assets: {e}");
        return ExitCode::FAILURE;
    }

    // Copy the binary to the output directory
    let bin = format!("{BIN_NAME}{}", std::env::consts::EXE_SUFFIX);
    info!("Copying {bin}...");
    let built_bin = cwd.join("target").join(profile.target_dir()).join(&bin);
    let out_bin = out_dir.join(&bin);
    if let Err(e) = fs::copy(&built_bin, &out_bin) {
        error!("Failed to copy {:?} to {:?}: {e}", built_bin, out_bin);
        return ExitCode::FAILURE;
    }

    // Start run
    info!("Running {bin}...");
    let run_status = match Command::new(&out_bin)
        .args(&flags)
        .current_dir(&out_dir)
        .status()
    {
        Ok(status) => status,
        Err(e) => {
            error!("Failed to start {bin}: {e}");
            return ExitCode::FAILURE;
        }
    };
    if !run_status.success() {
        error!("Run exited with {run_status}");
        return ExitCode::FAILURE;
    }

    info!("Run succeeded");
    ExitCode::SUCCESS
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
