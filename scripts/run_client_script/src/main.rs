use tracing::info;

fn main() {
    let _guard = runtime_logging::init_logging("Tbrun", "Client");

    let cwd = std::env::current_dir().unwrap();
    info!("cwd: {:?}", cwd);

    panic!("panik");

    let client_dir = cwd.join("").canonicalize().unwrap();
}
