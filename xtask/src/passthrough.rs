//! tasks that build a workspace binary with the `release` profile and run it, passing every
//! argument through. those binaries read their flags with `RuntimeFlags`, see their READMEs

use std::io;
use std::process::{Command, ExitStatus};

use clap::Args;
use snafu::{ResultExt, Snafu, ensure};
use tracing::info;

/// package name of the pack generator, run by `cargo xtask pkgen`
pub const TBPK_GENERATOR: &str = "tbpk_generator";
/// package name of the example generator, run by `cargo xtask regen`
pub const EXAMPLE_GENERATOR: &str = "example_generator";

#[derive(Debug, Args)]
pub struct PassthroughArgs {
    /// Flags passed to the binary as is, see its README
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

/// the binary couldn't be started or failed
#[derive(Debug, Snafu)]
pub enum PassthroughError {
    #[snafu(display("failed to start `{command}`, is cargo on PATH?"))]
    Start { command: String, source: io::Error },

    #[snafu(display("`{command}` failed with {status}, see its output above"))]
    Failed { command: String, status: ExitStatus },
}

#[tracing::instrument(skip_all, fields(package, args = ?args.args))]
pub fn run(package: &str, args: PassthroughArgs) -> Result<(), PassthroughError> {
    let mut cargo_args = vec!["run", "--profile", "release", "-p", package, "--"];
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
