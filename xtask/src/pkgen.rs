//! `cargo xtask pkgen`: builds `tbpk_generator` with the `release` profile and runs it,
//! passing every argument through. its flags are read with `RuntimeFlags`, see its README

use std::io;
use std::process::{Command, ExitStatus};

use clap::Args;
use snafu::{ResultExt, Snafu, ensure};
use tracing::info;

/// package name of the pack generator binary
const TBPK_GENERATOR: &str = "tbpk_generator";

#[derive(Debug, Args)]
pub struct PkgenArgs {
    /// Flags passed to tbpk_generator as is, like `--manifest <path>` (see bins/tbpk_generator/README.md)
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

/// tbpk_generator couldn't be started or failed
#[derive(Debug, Snafu)]
pub enum PkgenError {
    #[snafu(display("failed to start `{command}`, is cargo on PATH?"))]
    Start { command: String, source: io::Error },

    #[snafu(display("`{command}` failed with {status}, see its output above"))]
    Failed { command: String, status: ExitStatus },
}

#[tracing::instrument(skip_all, fields(args = ?args.args))]
pub fn run(args: PkgenArgs) -> Result<(), PkgenError> {
    let mut cargo_args = vec!["run", "--profile", "release", "-p", TBPK_GENERATOR, "--"];
    cargo_args.extend(args.args.iter().map(String::as_str));
    let command = format!("cargo {}", cargo_args.join(" "));

    info!("Running `{command}`...");
    let status = Command::new("cargo")
        .args(&cargo_args)
        .status()
        .context(StartSnafu { command: &command })?;
    ensure!(status.success(), FailedSnafu { command, status });
    Ok(())
}
