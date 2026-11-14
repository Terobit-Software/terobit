//! Workspace tasks, run with `cargo xtask <command>` (or `cargo x`) from the workspace root

mod run;

use std::process::ExitCode;

use clap::{Parser, Subcommand};
use snafu::Report;
use tracing::{error, info};

/// `(first, last)` names passed to `init_logging`
const LOG_NAME: (&str, &str) = ("Terobit", "Xtask");

/// Terobit workspace tasks
#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Build and run a binary
    #[command(subcommand)]
    Run(run::RunCommand),
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    // returning ExitCode (instead of process::exit) lets the guard drop and flush logs
    let _guard = runtime_logging::init_logging(LOG_NAME.0, LOG_NAME.1);

    let result = match cli.command {
        Command::Run(command) => run::run(command),
    };

    match result {
        Ok(()) => {
            info!("Task succeeded");
            ExitCode::SUCCESS
        }
        Err(e) => {
            error!("Task failed: {}", Report::from_error(&e));
            ExitCode::FAILURE
        }
    }
}
