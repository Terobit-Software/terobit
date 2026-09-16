#![cfg_attr(feature = "dist", windows_subsystem = "windows")]

use runtime::Runtime;
use std::path::{Path, PathBuf};
use tracing::{error, info};
use tracing_appender::rolling;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{Layer, fmt};

mod routine;
use routine::*;

mod listeners;
use listeners::*;

#[inline]
/// Returns the directory where log files should be stored
fn get_log_directory() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        Path::new(&format!(
            "/Users/{}/Library/Logs/TerobitClient/",
            std::env::var("USER").unwrap_or_default()
        ))
        .to_path_buf()
    }
    // these are placeholders
    #[cfg(target_os = "linux")]
    {
        Path::new("~/.local/share/terobit/logs/").to_path_buf()
    }
    #[cfg(target_os = "windows")]
    {
        let base = std::env::var("LOCALAPPDATA").expect("LOCALAPPDATA not set");
        PathBuf::from(base)
            .join("terobit")
            .join("client")
            .join("logs")
    }
}

/// The entrypoint of the client
fn main() {
    let log_dir = get_log_directory();
    std::fs::create_dir_all(&log_dir).unwrap();

    // daily rotation: creates terobit-client.log.2026-09-04, etc.
    let file_appender = rolling::daily(&log_dir, "terobit-client.log");
    // _guard must stay alive for the whole program or buffered logs get dropped
    let (writer, _guard) = tracing_appender::non_blocking(file_appender);

    // init subscriber
    let subscriber = tracing_subscriber::registry()
        // this one for terminal, color
        .with(fmt::layer().with_filter(LevelFilter::INFO))
        // this one for file, no color
        .with(
            fmt::layer()
                .with_writer(writer)
                .with_ansi(false)
                .with_filter(LevelFilter::INFO),
        );

    // this should be the LAST unwrap in the entire project
    // except where err or none is fatal as a result, but an error!() should be thrown before unwrap
    tracing::subscriber::set_global_default(subscriber).unwrap();

    info!("=========================");
    info!("=== CLIENT START HERE ===");
    info!("=========================");

    info!("Initializing runtime");

    // Initialize the runtime
    let mut runtime = Runtime::new();

    // Register frame routines
    runtime.add_routine(Routine {});

    // Register window listeners
    runtime.add_window_listener(ClientWindowListener {});

    // Start the runtime
    info!("Starting runtime");
    if let Err(e) = runtime.start() {
        error!("Failed to start runtime: {}", e);
        std::process::exit(1);
    } else {
        info!("Runtime exited successfully");
    }
}
