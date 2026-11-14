//! Workspace tasks, run with `cargo xtask <command>` (or `cargo x`) from the workspace root

mod passthrough;
mod run;

use std::process::ExitCode;

use clap::{Parser, Subcommand};
use snafu::{Report, ResultExt, Snafu};
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
    #[command(subcommand, visible_alias = "r")]
    Run(run::RunCommand),

    /// Build .tbpk packs with tbpk_generator
    Pkgen(passthrough::PassthroughArgs),

    /// Write generated example dev directories with example_generator
    Regen(passthrough::PassthroughArgs),
}

/// a task failed
#[derive(Debug, Snafu)]
enum TaskError {
    #[snafu(display("run failed"))]
    Run { source: run::RunError },

    #[snafu(display("{package} failed"))]
    Passthrough {
        package: &'static str,
        source: passthrough::PassthroughError,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    // returning ExitCode (instead of process::exit) lets the guard drop and flush logs
    let _guard = runtime_logging::init_logging(LOG_NAME.0, LOG_NAME.1);

    let result = match cli.command {
        Command::Run(command) => run::run(command).context(RunSnafu),
        Command::Pkgen(args) => passthrough(passthrough::TBPK_GENERATOR, args),
        Command::Regen(args) => passthrough(passthrough::EXAMPLE_GENERATOR, args),
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

fn passthrough(package: &'static str, args: passthrough::PassthroughArgs) -> Result<(), TaskError> {
    passthrough::run(package, args).context(PassthroughSnafu { package })
}
