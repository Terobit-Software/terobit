use runtime::Runtime;
use std::fs::File;
use tracing::{error, info};
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{Layer, fmt};

fn main() {
    // create a file for logging
    let log_file = File::create(format!(
        "terobit-client-{}.log",
        uuid::Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext))
    ))
    .unwrap();

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
        error!("Failed to initialize runtime: {}", e);
        std::process::exit(1);
    } else {
        info!("Runtime initialized successfully");
    }
}
