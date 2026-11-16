//! builds a dev directory's scripts before they're packed or run.
//!
//! a script path like `scripts/client/main.wasm` can have a source folder next to it, the
//! same path without `.wasm` (`scripts/client/main/`). if that folder has a `build.sh`, it's
//! run from inside the folder as `sh build.sh` (`sh build.sh --final` for final builds) and
//! must write `script.wasm` into the folder, which is then copied over `main.wasm`. without
//! the folder, the `.wasm` already there is used as is

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

use runtime_assets::ManifestSource;
use runtime_assets::pack::Target;
use runtime_consts::pack::SCRIPT_EXTENSION;
use snafu::{ResultExt, Snafu, ensure};
use tracing::{info, warn};

/// script that builds a script's source folder
pub const BUILD_SCRIPT: &str = "build.sh";

/// file `BUILD_SCRIPT` writes into the source folder
pub const BUILD_OUTPUT: &str = "script.wasm";

/// flag passed to `BUILD_SCRIPT` for final builds
pub const FINAL_FLAG: &str = "--final";

/// a script couldn't be built or found
#[derive(Debug, Snafu)]
pub enum ScriptError {
    #[snafu(display("failed to remove the old build output `{}`", path.display()))]
    RemoveOld { path: PathBuf, source: io::Error },

    #[snafu(display("failed to start `{command}` in `{}`, is `sh` on PATH?", dir.display()))]
    StartBuild {
        command: String,
        dir: PathBuf,
        source: io::Error,
    },

    #[snafu(display(
        "`{command}` in `{}` failed with {status}, see its output above",
        dir.display()
    ))]
    BuildFailed {
        command: String,
        dir: PathBuf,
        status: ExitStatus,
    },

    #[snafu(display(
        "`{command}` in `{}` succeeded but didn't write `{}`, {BUILD_SCRIPT} must write its output there",
        dir.display(),
        output.display()
    ))]
    NoOutput {
        command: String,
        dir: PathBuf,
        output: PathBuf,
    },

    #[snafu(display("failed to copy the built script from `{}` to `{}`", from.display(), to.display()))]
    CopyOutput {
        from: PathBuf,
        to: PathBuf,
        source: io::Error,
    },

    #[snafu(display(
        "{target:?} script `{id}` points to `{}`, which doesn't exist, and there's no `{}` folder with a {BUILD_SCRIPT} to build it",
        wasm.display(),
        dir.display()
    ))]
    Missing {
        target: Target,
        id: String,
        wasm: PathBuf,
        dir: PathBuf,
    },
}

/// builds every script one side of the manifest runs, see the module doc. stops at the
/// first script that fails
#[tracing::instrument(skip_all, fields(?target, is_final))]
pub fn build_side(
    source: &ManifestSource,
    target: Target,
    is_final: bool,
) -> Result<(), ScriptError> {
    let root = source.get_root();
    for (id, path) in source.scripts_for(target) {
        let wasm = root.join(path);
        // the manifest only accepts paths ending in SCRIPT_EXTENSION
        let dir = root.join(path.strip_suffix(SCRIPT_EXTENSION).unwrap_or(path));
        let build_sh = dir.join(BUILD_SCRIPT);

        if build_sh.is_file() {
            build_one(&dir, &wasm, is_final)?;
            continue;
        }
        if dir.is_dir() {
            warn!(
                "{target:?} script `{id}` has a source folder `{}` but no {BUILD_SCRIPT} in it, using `{}` as is",
                dir.display(),
                wasm.display()
            );
        }
        ensure!(
            wasm.is_file(),
            MissingSnafu {
                target,
                id,
                wasm,
                dir
            }
        );
        info!(
            "{target:?} script `{id}` has no source folder, using `{}` as is",
            wasm.display()
        );
    }
    Ok(())
}

/// runs `BUILD_SCRIPT` in `dir` and copies its output to `wasm`
fn build_one(dir: &Path, wasm: &Path, is_final: bool) -> Result<(), ScriptError> {
    // a stale output would hide a build.sh that stopped writing one
    let output = dir.join(BUILD_OUTPUT);
    if output.exists() {
        fs::remove_file(&output).context(RemoveOldSnafu { path: &output })?;
    }

    let mut args = vec![BUILD_SCRIPT];
    if is_final {
        args.push(FINAL_FLAG);
    }
    let command = format!("sh {}", args.join(" "));
    info!("Building script with `{command}` in {}...", dir.display());
    let status = Command::new("sh")
        .args(&args)
        .current_dir(dir)
        .status()
        .context(StartBuildSnafu {
            command: &command,
            dir,
        })?;
    ensure!(
        status.success(),
        BuildFailedSnafu {
            command,
            dir,
            status
        }
    );
    ensure!(
        output.is_file(),
        NoOutputSnafu {
            command,
            dir,
            output
        }
    );

    fs::copy(&output, wasm).context(CopyOutputSnafu {
        from: &output,
        to: wasm,
    })?;
    info!("Copied {} to {}", output.display(), wasm.display());
    Ok(())
}
