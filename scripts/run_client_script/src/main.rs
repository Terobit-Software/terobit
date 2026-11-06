use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, ExitStatus};

use client_consts::flags::{DEV, PACK_FILE};
use client_consts::paths::{CLIENT_BIN, CLIENT_OUT};
use runtime_run_flags::RuntimeFlags;
use snafu::{Report, ResultExt, Snafu, ensure};
use tracing::{error, info, warn};

/// pack loaded when `--pack-file` isn't given, relative to the workspace root
const DEFAULT_PACK_FILE: &str = "tests/examples/source_to_tbpk/basic1";

/// Build and run the terobit client
#[derive(Debug)]
struct Args {
    /// Build in final (release) mode
    r#final: bool,
    /// Don't pass the default dev flags to the client
    strip_defaults: bool,
    /// pack or dev directory the client loads, relative to the current directory
    pack_file: PathBuf,
}

/// run flags that don't make a valid set of `Args`
#[derive(Debug, Snafu)]
enum ArgsError {
    #[snafu(display(
        "unknown flag {flag}, expected --final, --strip-defaults or {PACK_FILE} <path>"
    ))]
    UnknownFlag { flag: String },

    #[snafu(display(
        "`{arg}` isn't a flag, flags are long only (`-f` is now `--final`, `-s` is `--strip-defaults`)"
    ))]
    NotAFlag { arg: String },

    #[snafu(display("{flag} doesn't take a value, but was given {values:?}"))]
    TakesNoValue { flag: String, values: Vec<String> },

    #[snafu(display("{PACK_FILE} takes exactly one path, but was given {}: {values:?}", values.len()))]
    PackFileCount { values: Vec<String> },

    #[snafu(display("{PACK_FILE} only applies to dev runs, it can't be used with {flag}"))]
    PackFileUnused { flag: &'static str },
}

impl Args {
    /// `RuntimeFlags` drops anything before the first `--flag`, so check for it here instead of
    /// silently running with defaults
    fn from_args(args: Vec<String>) -> Result<Self, ArgsError> {
        if let Some(arg) = args.iter().take_while(|arg| !arg.starts_with("--")).next() {
            return NotAFlagSnafu { arg }.fail();
        }
        Self::from_flags(&RuntimeFlags::from_args(args))
    }

    fn from_flags(flags: &RuntimeFlags) -> Result<Self, ArgsError> {
        let flags = flags.get_flags();
        for flag in flags.keys() {
            ensure!(
                ["--final", "--strip-defaults", PACK_FILE].contains(&flag.as_str()),
                UnknownFlagSnafu { flag }
            );
        }

        let switch = |flag: &str| -> Result<bool, ArgsError> {
            match flags.get(flag) {
                None => Ok(false),
                Some(values) if values.is_empty() => Ok(true),
                Some(values) => TakesNoValueSnafu {
                    flag,
                    values: values.clone(),
                }
                .fail(),
            }
        };
        let r#final = switch("--final")?;
        let strip_defaults = switch("--strip-defaults")?;

        let pack_file = match flags.get(PACK_FILE) {
            None => PathBuf::from(DEFAULT_PACK_FILE),
            Some(values) => {
                let [path] = values.as_slice() else {
                    return PackFileCountSnafu {
                        values: values.clone(),
                    }
                    .fail();
                };
                ensure!(!r#final, PackFileUnusedSnafu { flag: "--final" });
                ensure!(
                    !strip_defaults,
                    PackFileUnusedSnafu {
                        flag: "--strip-defaults"
                    }
                );
                PathBuf::from(path)
            }
        };

        Ok(Self {
            r#final,
            strip_defaults,
            pack_file,
        })
    }
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

/// the client couldn't be built, staged or run
#[derive(Debug, Snafu)]
enum RunError {
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
}

fn main() -> ExitCode {
    // returning ExitCode (instead of process::exit) lets the guard drop and flush logs
    let (first, last) = client_consts::logging::RUN_CLIENT_SCRIPT;
    let _guard = runtime_logging::init_logging(first, last);

    let args = match Args::from_args(std::env::args().skip(1).collect()) {
        Ok(args) => args,
        Err(e) => {
            error!("Invalid run client flags: {}", Report::from_error(&e));
            return ExitCode::FAILURE;
        }
    };

    match run(args) {
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

    let client_dir = cwd.join("bins").join(CLIENT_BIN);
    info!("client dir: {:?}", client_dir.canonicalize());

    let assets_dir = client_dir.join("assets");
    info!("assets dir: {:?}", assets_dir.canonicalize());

    let out_dir = cwd.join(CLIENT_OUT);
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
    let pack_file = cwd.join(&args.pack_file).to_string_lossy().into_owned();
    info!("pack file: {pack_file}");

    let mut flags: Vec<&str> = vec![];
    if !is_final && !args.strip_defaults {
        flags.extend([DEV, PACK_FILE, &pack_file]);
    }
    info!("client flags: {:?}", flags);

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
    let bin = format!("{}{}", CLIENT_BIN, std::env::consts::EXE_SUFFIX);
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
