use bevy::ecs::system::Commands;
use bevy::ecs::{
    event::Event,
    observer::On,
    system::Res,
    world::{Mut, World},
};
use bevy::time::Time;
use runtime_assets::script_imports::{ScriptImportError, check_script_imports};
use runtime_assets::{PackReadEvent, pack::AssetHandle, registry::PackRegistry};
use runtime_consts::sandbox::WASM_CYCLE_TICKS;

use crate::HostState;
use crate::host::state::with_world;

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
    #[snafu(display("refusing to load module `{handle}`"))]
    Imports {
        handle: AssetHandle,
        source: ScriptImportError,
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

    let bytes = std::fs::read(full).context(ReadSnafu {
        handle: handle.clone(),
    })?;

    // packs are checked when they're written, but a pack can come from anywhere
    check_script_imports(&bytes).context(ImportsSnafu {
        handle: handle.clone(),
    })?;

    Ok(bytes)
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
    mut commands: Commands,
) {
    let handle = event.handle.clone();
    tracing::info!("New load module request from {}", handle);

    match get_module_bytes(&handle, &registry) {
        Ok(bytes) => commands.queue(move |world: &mut World| load_module(world, handle, bytes)),
        Err(e) => tracing::error!("{}", snafu::Report::from_error(&e)),
    }
}

fn load_module(world: &mut World, handle: AssetHandle, bytes: Vec<u8>) {
    let loaded_at = world.resource::<Time>().elapsed();

    world.resource_scope(|world, mut sandbox: Mut<crate::RuntimeSandboxResource>| {
        let crate::RuntimeSandboxResource {
            engine,
            instances,
            linker,
        } = &mut *sandbox;

        let module = match wasmtime::Module::new(engine, &bytes) {
            Ok(m) => m,
            Err(e) => {
                tracing::error!("Failed to compile module `{}`: {:?}", handle, e);
                return;
            }
        };

        // every script gets its own store, so scripts can't see or corrupt each other's state
        let mut store = wasmtime::Store::new(engine, HostState::default());

        // instantiation may run the module's start function, so give it a cycle budget
        store.set_epoch_deadline(WASM_CYCLE_TICKS);

        let v = match with_world(&mut store, world, |store| {
            linker.instantiate(store, &module)
        }) {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("Failed to instantiate module `{}`: {:?}", handle, e);
                return;
            }
        };

        tracing::info!("Loaded module `{}`", handle);
        let mut script = crate::ScriptInstance {
            instance: v,
            store,
            loaded_at,
            frame: 0,
            poisoned: false,
        };

        match with_world(&mut script.store, world, |store| prep_instance(&v, store)) {
            Ok(()) => {
                if let Err(e) =
                    crate::commands::read_and_apply_commands(&v, &mut script.store, world)
                {
                    tracing::error!("load module failed: {:?}", e);
                    script.poison(&format!("error while loading buffer: {:?}", e));
                };
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

        instances.push(script);
    });
}
