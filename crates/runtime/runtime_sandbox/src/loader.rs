use bevy::ecs::{event::Event, observer::On, system::Res};
use runtime_assets::{pack::AssetHandle, registry::PackRegistry};

#[derive(Event)]
pub struct LoadModuleEvent {
    pub handle: AssetHandle,
}

use snafu::{OptionExt, ResultExt, Snafu};
use std::path::Path;

#[derive(Debug, Snafu)]
pub enum ModuleLoadError {
    #[snafu(display("no loaded pack declares module `{handle}`"))]
    UnknownHandle { handle: AssetHandle },
    #[snafu(display("module `{handle}` resolves to `{path}`, which isn't in the pack"))]
    MissingFile { handle: AssetHandle, path: String },
    #[snafu(display("failed to read module `{handle}`"))]
    Read {
        handle: AssetHandle,
        source: std::io::Error,
    },
}

fn get_module_bytes(
    handle: &AssetHandle,
    registry: &PackRegistry,
) -> Result<Vec<u8>, ModuleLoadError> {
    let path = registry.resolve(handle).context(UnknownHandleSnafu {
        handle: handle.clone(),
    })?;

    let full = registry
        .full_path(Path::new(&path))
        .context(MissingFileSnafu {
            handle: handle.clone(),
            path: path.clone(),
        })?;

    std::fs::read(full).context(ReadSnafu {
        handle: handle.clone(),
    })
}

pub fn observe_load_module_event(event: On<LoadModuleEvent>, registry: Res<PackRegistry>) {
    let handle = event.handle.clone();
    tracing::info!("New load module request from {}", handle);

    let bytes = match get_module_bytes(&handle, &registry) {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("{}", snafu::Report::from_error(&e));
            return;
        }
    };
}
