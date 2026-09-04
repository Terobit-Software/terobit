use runtime::Runtime;
use std::fs::File;
use std::path::{Path, PathBuf};
use tracing::{error, info};
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{Layer, fmt};

#[inline]
/// Returns the directory where log files should be stored
fn get_log_directory() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        return Path::new(&format!(
            "/Users/{}/Library/Logs/TerobitClient/",
            std::env::var("USER").unwrap_or_default()
        ))
        .to_path_buf();
    }
    // these are placeholders
    #[cfg(target_os = "linux")]
    {
        return Path::new("~/.local/share/terobit/logs/").to_path_buf();
    }
    #[cfg(target_os = "windows")]
    {
        return Path::new("C:\\Users\\").to_path_buf();
    }
}

/// The entrypoint of the client
fn main() {
    let log_dir = get_log_directory();
    std::fs::create_dir_all(&log_dir).unwrap();

    let log_file = File::create_new(log_dir.join(format!(
        "terobit-client-{}.log",
        uuid::Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext))
    )))
    .unwrap();

    println!("{:?}", log_file);

    // init subscriber
    let subscriber = tracing_subscriber::registry()
        // this one for terminal, color
        .with(fmt::layer().with_filter(LevelFilter::INFO))
        // this one for file, no color
        .with(
            fmt::layer()
                .with_writer(log_file)
                .with_ansi(false)
                .with_filter(LevelFilter::INFO),
        );

    // this should be the LAST unwrap in the entire project
    // except where err or none is fatal as a result, but an error!() should be thrown before unwrap
    tracing::subscriber::set_global_default(subscriber).unwrap();

    info!("Initializing runtime");

    // Initialize the runtime
    let ref runtime = Runtime::new();

    // Start the runtime
    info!("Starting runtime");
    if let Err(e) = runtime.start() {
        error!("Failed to start runtime: {}", e);
        std::process::exit(1);
    } else {
        info!("Runtime exited successfully");
    }
}
