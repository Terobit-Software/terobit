use runtime::Runtime;
use tracing::{Level, error, info};
use tracing_subscriber::FmtSubscriber;

fn main() {
    // Initialize the tracing subscriber
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    // Initialize the runtime
    let ref runtime = Runtime::new();

    // Check if the runtime initialized successfully
    if let Err(e) = runtime.init() {
        error!("Failed to initialize runtime: {}", e);
        std::process::exit(1);
    } else {
        info!("Runtime initialized successfully");
    }
}
