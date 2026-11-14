//! `cargo xtask run <target>`

mod client;

use clap::Subcommand;
use snafu::{ResultExt, Snafu};

#[derive(Debug, Subcommand)]
pub enum RunCommand {
    /// Build the client and run it from a clean `out/client`
    #[command(visible_alias = "cl")]
    Client(client::ClientArgs),
}

/// a `run` task failed
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
