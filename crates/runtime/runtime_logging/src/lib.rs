use snafu::{Report, ResultExt, Snafu};
use std::path::PathBuf;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{Layer, fmt};

/// the log file couldn't be set up, logging falls back to the terminal only
#[derive(Debug, Snafu)]
pub enum LoggingError {
    #[snafu(display(
        "environment variable `{var}` is needed to find the log directory, but it isn't set or isn't valid unicode"
    ))]
    MissingEnv {
        var: &'static str,
        source: std::env::VarError,
    },

    #[snafu(display("failed to create log directory `{}`", path.display()))]
    CreateLogDir {
        path: PathBuf,
        source: std::io::Error,
    },
}

fn env(var: &'static str) -> Result<String, LoggingError> {
    std::env::var(var).context(MissingEnvSnafu { var })
}

#[inline]
/// Returns the directory where log files should be stored, `<first>/<last>` under each OS's
/// usual folder. capitalized on macOS, lowercase elsewhere
fn get_log_directory(
    first_name: &'static str,
    last_name: &'static str,
) -> Result<PathBuf, LoggingError> {
    #[cfg(target_os = "macos")]
    {
        Ok(
            PathBuf::from(format!("/Users/{}/Library/Logs", env("USER")?))
                .join(first_name)
                .join(last_name),
        )
    }
    // $XDG_DATA_HOME, falling back to ~/.local/share per the XDG base directory spec
    #[cfg(target_os = "linux")]
    {
        let base = match std::env::var("XDG_DATA_HOME")
            .ok()
            .filter(|dir| !dir.is_empty())
        {
            Some(dir) => PathBuf::from(dir),
            None => PathBuf::from(env("HOME")?).join(".local").join("share"),
        };
        Ok(base
            .join(first_name.to_lowercase())
            .join(last_name.to_lowercase())
            .join("logs"))
    }
    #[cfg(target_os = "windows")]
    {
        Ok(PathBuf::from(env("LOCALAPPDATA")?)
            .join(first_name.to_lowercase())
            .join(last_name.to_lowercase())
            .join("logs"))
    }
}

/// `<first>-<last>.log`, lowercase
fn log_file_name(first_name: &str, last_name: &str) -> String {
    format!(
        "{}-{}.log",
        first_name.to_lowercase(),
        last_name.to_lowercase()
    )
}

fn panic_hook(info: &std::panic::PanicHookInfo<'_>) {
    let payload = info.payload();

    // Extract the panic message if it's a string slice or an owned String
    let message = if let Some(s) = payload.downcast_ref::<&str>() {
        *s
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.as_str()
    } else {
        "Box<dyn Any>"
    };

    let location = info
        .location()
        .map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()))
        .unwrap_or_else(|| "unknown location".to_string());

    // Emit standard tracing error event
    tracing::error!(target: "panic", message = %message, location = %location, "A PANIC occurred");
}

/// creates the log directory and a daily rolling, non blocking writer into it
fn file_writer(
    first_name: &'static str,
    last_name: &'static str,
) -> Result<
    (
        tracing_appender::non_blocking::NonBlocking,
        WorkerGuard,
        PathBuf,
    ),
    LoggingError,
> {
    let log_dir = get_log_directory(first_name, last_name)?;
    std::fs::create_dir_all(&log_dir).context(CreateLogDirSnafu { path: &log_dir })?;

    // daily rotation: creates terobit-client.log.2026-09-04, etc.
    let file_appender = rolling::daily(&log_dir, log_file_name(first_name, last_name));
    let (writer, guard) = tracing_appender::non_blocking(file_appender);
    Ok((writer, guard, log_dir))
}

/// both names should be formatted as `Name`, NOT `name` or `NAME`. Only the first letter should be
/// capitalized, macOS log folders use them as given
///
/// never fails: if the log file can't be set up, logs only go to the terminal and a warning
/// says why. the returned guard must stay alive for the whole program or buffered file logs
/// get dropped, it is `None` when there is no log file
pub fn init_logging(first_name: &'static str, last_name: &'static str) -> Option<WorkerGuard> {
    let (file_layer, guard, file_result) = match file_writer(first_name, last_name) {
        Ok((writer, guard, log_dir)) => (
            // no color in the file
            Some(
                fmt::layer()
                    .with_writer(writer)
                    .with_ansi(false)
                    .with_filter(LevelFilter::INFO),
            ),
            Some(guard),
            Ok(log_dir),
        ),
        Err(e) => (None, None, Err(e)),
    };

    // init subscriber
    let subscriber = tracing_subscriber::registry()
        // this one for terminal, color
        .with(fmt::layer().with_filter(LevelFilter::INFO))
        .with(file_layer);

    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        panic_hook(panic_info);
        prev_hook(panic_info);
    }));

    // only fails if a subscriber is already set, which then gets these logs instead
    if let Err(e) = tracing::subscriber::set_global_default(subscriber) {
        tracing::warn!(
            "Logging for {first_name}{last_name} wasn't set up because a global tracing subscriber is already set, logs go to that one instead: {}",
            Report::from_error(&e)
        );
    }

    match file_result {
        Ok(log_dir) => tracing::info!("Logging to the terminal and {}", log_dir.display()),
        Err(e) => tracing::warn!(
            "Logging to the terminal only, logs from this run won't be saved to a file: {}",
            Report::from_error(&e)
        ),
    }

    guard
}

#[cfg(test)]
mod fn_tests {
    use super::*;

    #[test]
    #[cfg(target_os = "macos")]
    fn log_directory_macos() {
        let user = std::env::var("USER").unwrap();

        assert_eq!(
            get_log_directory("Terobit", "Client").unwrap(),
            PathBuf::from(format!("/Users/{}/Library/Logs/Terobit/Client", user))
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
        let dir = get_log_directory("Terobit", "Client").unwrap();

        assert!(dir.is_absolute());
        assert!(!dir.to_string_lossy().contains('~'));
        assert_eq!(dir, base.join("terobit").join("client").join("logs"));
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn log_directory_windows() {
        let base = std::env::var("LOCALAPPDATA").unwrap();

        assert_eq!(
            get_log_directory("Terobit", "Client").unwrap(),
            PathBuf::from(base)
                .join("terobit")
                .join("client")
                .join("logs")
        );
    }

    #[test]
    fn log_file_is_named_after_the_app() {
        assert_eq!(log_file_name("Terobit", "Client"), "terobit-client.log");
        assert_eq!(log_file_name("Tbpk", "Generator"), "tbpk-generator.log");
    }

    #[test]
    fn log_directory_differs_per_app() {
        assert_ne!(
            get_log_directory("Terobit", "Client").unwrap(),
            get_log_directory("Tbpk", "Generator").unwrap()
        );
    }
}
