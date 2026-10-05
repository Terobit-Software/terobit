use std::path::PathBuf;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{Layer, fmt};

#[inline]
/// Returns the directory where log files should be stored
fn get_log_directory(first_name: &'static str, last_name: &'static str) -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        PathBuf::from(format!(
            "/Users/{}/Library/Logs/{}{}/",
            std::env::var("USER").unwrap(),
            first_name,
            last_name
        ))
    }
    // $XDG_DATA_HOME, falling back to ~/.local/share per the XDG base directory spec
    #[cfg(target_os = "linux")]
    {
        std::env::var("XDG_DATA_HOME")
            .ok()
            .filter(|dir| !dir.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(std::env::var("HOME").expect("HOME not set"))
                    .join(".local")
                    .join("share")
            })
            .join(format!(
                "{}{}",
                first_name.to_lowercase(),
                last_name.to_lowercase()
            ))
            .join("logs")
    }
    #[cfg(target_os = "windows")]
    {
        let base = std::env::var("LOCALAPPDATA").expect("LOCALAPPDATA not set");
        PathBuf::from(base)
            .join(first_name.to_lowercase())
            .join(last_name.to_lowercase())
            .join("logs")
    }
}

/// first name should be formated as `Name` NOT `name` or `NAME`. Only the first letter should be capitalized
pub fn init_logging(first_name: &'static str, last_name: &'static str) -> WorkerGuard {
    let log_dir = get_log_directory(first_name, last_name);
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

    _guard
}

#[cfg(test)]
mod fn_tests {
    use super::*;

    #[test]
    #[cfg(target_os = "macos")]
    fn log_directory_macos() {
        let user = std::env::var("USER").unwrap();

        assert_eq!(
            get_log_directory("Terobit", "Client"),
            PathBuf::from(format!("/Users/{}/Library/Logs/TerobitClient/", user))
        );
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn log_directory_linux() {
        let base = std::env::var("XDG_DATA_HOME")
            .ok()
            .filter(|dir| !dir.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(std::env::var("HOME").unwrap()).join(".local/share"));
        let dir = get_log_directory("Terobit", "Client");

        assert!(dir.is_absolute());
        assert!(!dir.to_string_lossy().contains('~'));
        assert_eq!(dir, base.join("terobitclient").join("logs"));
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn log_directory_windows() {
        let base = std::env::var("LOCALAPPDATA").unwrap();

        assert_eq!(
            get_log_directory("Terobit", "Client"),
            PathBuf::from(base)
                .join("terobit")
                .join("client")
                .join("logs")
        );
    }

    #[test]
    fn log_directory_differs_per_app() {
        assert_ne!(
            get_log_directory("Terobit", "Client"),
            get_log_directory("Tbpk", "Generator")
        );
    }
}
