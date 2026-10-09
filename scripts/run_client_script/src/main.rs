use clap::Parser;
use tracing::info;

/// Build and run the terobit client
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Build in final (release) mode
    #[arg(short, long)]
    r#final: bool,
}

#[derive(Debug)]
enum BuildProfile {
    Release,
    Dev,
    ReleaseDev,
}

fn main() {
    let _guard = runtime_logging::init_logging("Tbrun", "Client");

    let args = Args::parse();

    let cwd = std::env::current_dir().unwrap();
    info!("cwd: {:?}", cwd);

    let client_dir = cwd.join("bins/terobit_client");
    info!("client dir: {:?}", client_dir.canonicalize());

    let assets_dir = client_dir.join("assets");
    info!("assets dir: {:?}", assets_dir.canonicalize());

    let out_dir = cwd.join("out/client");
    info!("out dir: {:?}", out_dir.canonicalize());

    let is_final = args.r#final;
    info!("final: {is_final}");

    let profile = if is_final {
        BuildProfile::Release
    } else {
        BuildProfile::ReleaseDev
    };
    info!("profile: {:?}", profile);
}
