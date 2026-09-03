use runtime::Runtime;
use tracing::{Level, error, info};
use tracing_subscriber::FmtSubscriber;

fn main() {
    // Initialize the tracing subscriber
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    // this should be the ONLY unwrap in the entire project
    // except where err or none is fatal as a result
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
