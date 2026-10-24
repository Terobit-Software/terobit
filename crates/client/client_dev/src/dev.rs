use bevy::prelude::*;
use runtime_assets::pack::Target;
use runtime_assets::{LoadPackFileEvent, RuntimePackConfig};
use runtime_run_flags::RuntimeFlags;
use std::path::{Path, PathBuf};
use tracing::{error, info, warn};

use crate::DeveloperModeStatus;

/// lets the client read dev directories and loads `--pack-file`
pub(crate) fn enable(app: &mut App) {
    info!(
        "\n\n--------------------------------\nClient running in DEVELOPER mode\n--------------------------------\n"
    );
    app.world_mut()
        .insert_resource(DeveloperModeStatus { value: true });

    if let Some(mut config) = app.world_mut().get_resource_mut::<RuntimePackConfig>() {
        config.set_allow_dir_read(true);
    } else {
        warn!(
            "Developer mode couldn't allow reading dev directories as packs, the RuntimePackConfig resource doesn't exist. Is RuntimePackReaderPlugin added before ClientDevPlugin?"
        )
    }

    match pack_file(app.world().resource::<RuntimeFlags>()) {
        Some(path) => {
            app.world_mut().trigger(LoadPackFileEvent {
                path,
                target: Target::Client,
            });
        }
        None => warn!(
            "Developer mode is on but no pack will be loaded, pass `--pack-file <path to .tbpk or dev directory>` to load one"
        ),
    }
}

/// the first `--pack-file` path
fn pack_file(flags: &RuntimeFlags) -> Option<Box<Path>> {
    let values = flags.get_flags().get("--pack-file")?;
    match values.as_slice() {
        [] => {
            error!(
                "--pack-file was given without a path, expected `--pack-file <path to .tbpk or dev directory>`. No pack will be loaded"
            );
            None
        }
        [first, rest @ ..] => {
            if !rest.is_empty() {
                warn!(
                    "--pack-file was given {} paths, only the first (`{first}`) is loaded, ignoring {rest:?}",
                    values.len()
                );
            }
            Some(PathBuf::from(first).into_boxed_path())
        }
    }
}
