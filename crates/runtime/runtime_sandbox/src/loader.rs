use bevy::ecs::system::Commands;
use bevy::ecs::{
    event::Event,
    observer::On,
    system::{Res, ResMut},
};
use bevy::time::Time;
use runtime_assets::{PackReadEvent, pack::AssetHandle, registry::PackRegistry};
use runtime_consts::sandbox::WASM_CYCLE_TICKS;

#[derive(Event)]
pub struct LoadModuleEvent {
    pub handle: AssetHandle,
}

use snafu::{OptionExt, ResultExt, Snafu};
use std::path::Path;

mod prep;
pub use prep::prep_instance;

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
    let path = registry
        .resolve_script(handle)
        .context(UnknownHandleSnafu {
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

/// loads every script of a freshly read pack
pub fn observe_pack_read_event(
    event: On<PackReadEvent>,
    registry: Res<PackRegistry>,
    mut commands: Commands,
) {
    let handles = registry.script_handles();
    tracing::info!(
        "Pack read for {:?}, loading {} scripts",
        event.target,
        handles.len()
    );

    for handle in handles {
        commands.trigger(LoadModuleEvent { handle });
    }
}

pub fn observe_load_module_event(
    event: On<LoadModuleEvent>,
    registry: Res<PackRegistry>,
    time: Res<Time>,
    mut _sand: ResMut<crate::RuntimeSandboxResource>,
    mut commands: Commands,
) {
    let handle = event.handle.clone();
    tracing::info!("New load module request from {}", handle);

    let bytes = match get_module_bytes(&handle, &registry) {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("{}", snafu::Report::from_error(&e));
            return;
        }
    };

    let sandbox = &mut *_sand;

    let module = match wasmtime::Module::new(&sandbox.engine, &bytes) {
        Ok(m) => m,
        Err(e) => {
            tracing::error!("Failed to compile module `{}`: {:?}", handle, e);
            return;
        }
    };

    // instantiation may run the module's start function, so give it a cycle budget
    sandbox.store.set_epoch_deadline(WASM_CYCLE_TICKS);

    match sandbox.linker.instantiate(&mut sandbox.store, &module) {
        Ok(v) => {
            tracing::info!("Loaded module `{}`", handle);
            let mut script = crate::ScriptInstance {
                instance: v,
                loaded_at: time.elapsed(),
                frame: 0,
                poisoned: false,
            };

            match prep_instance(&v, &mut sandbox.store) {
                Ok(()) => {
                    crate::commands::read_and_apply_commands(&v, &mut sandbox.store, &mut commands);
                }
                // main never ran, so the guest is clean; nothing was queued
                Err(e @ prep::InstancePrepError::NoMainEntryPoint { .. }) => {
                    tracing::error!(
                        "Failed to prep module `{}`: {}",
                        handle,
                        snafu::Report::from_error(&e)
                    );
                }
                // main trapped partway: drop whatever it queued and stop running it
                Err(e @ prep::InstancePrepError::MainFailed { .. }) => {
                    tracing::error!(
                        "Failed to prep module `{}`: {}",
                        handle,
                        snafu::Report::from_error(&e)
                    );
                    script.poison(&format!("main of `{}` trapped", handle));
                }
            }

            sandbox.instances.push(script);
        }
        Err(e) => {
            tracing::error!("Failed to instantiate module `{}`: {:?}", handle, e);
        }
    };
}
