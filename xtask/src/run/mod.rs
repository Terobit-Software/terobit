//! `cargo xtask run <target>` and `cargo xtask runbuild <target>`

mod client;

use clap::Subcommand;
use snafu::{ResultExt, Snafu};

#[derive(Debug, Subcommand)]
pub enum RunCommand {
    /// Build the client and run it from a clean `out/client`
    #[command(visible_alias = "cl")]
    Client(client::ClientArgs),
}

#[derive(Debug, Subcommand)]
pub enum RunBuildCommand {
    /// Build the client scripts of a pack, optionally package it, then run the
    /// client on it
    #[command(visible_alias = "cl")]
    Client(client::RunBuildClientArgs),
}

/// a `run` or `runbuild` task failed
#[derive(Debug, Snafu)]
pub enum RunError {
    #[snafu(display("failed to run the client"))]
    Client { source: client::ClientError },
}

pub fn run(command: RunCommand) -> Result<(), RunError> {
    match command {
        RunCommand::Client(args) => client::run(args).context(ClientSnafu),
    }
}

pub fn run_build(command: RunBuildCommand) -> Result<(), RunError> {
    match command {
        RunBuildCommand::Client(args) => client::run_build(args).context(ClientSnafu),
    }
}
