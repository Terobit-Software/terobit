use tracing::info;

fn main() {
    let _guard = runtime_logging::init_logging("Tbrun", "Client");

    let cwd = std::env::current_dir().unwrap();
    info!("cwd: {:?}", cwd);

    let client_dir = cwd.join("bins/terobit_client").canonicalize().unwrap();
    info!("client dir: {:?}", client_dir);

    let assets_dir = client_dir.join("assets").canonicalize().unwrap();
    info!("assets dir: {:?}", assets_dir);
}
