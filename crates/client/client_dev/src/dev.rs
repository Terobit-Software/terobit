use bevy::prelude::*;
use client_consts::flags::PACK_FILE;
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
            "Developer mode is on but no pack will be loaded, pass `{} <path to .tbpk or dev directory>` to load one",
            PACK_FILE
        ),
    }
}

/// the first `PACK_FILE` path
fn pack_file(flags: &RuntimeFlags) -> Option<Box<Path>> {
    let values = flags.get_flags().get(PACK_FILE)?;
    match values.as_slice() {
        [] => {
            error!(
                "{} was given without a path, expected `{} <path to .tbpk or dev directory>`. No pack will be loaded",
                PACK_FILE, PACK_FILE
            );
            None
        }
        [first, rest @ ..] => {
            if !rest.is_empty() {
                warn!(
                    "{} was given {} paths, only the first (`{first}`) is loaded, ignoring {rest:?}",
                    PACK_FILE,
                    values.len()
                );
            }
            Some(PathBuf::from(first).into_boxed_path())
        }
    }
}
