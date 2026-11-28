use bevy::{
    ecs::resource::Resource,
    prelude::{App, First, IntoScheduleConfigs, Plugin, Update},
};
use runtime_consts::sandbox::WASM_EPOCH_TICK;

pub struct RuntimeSandboxPlugin;

pub mod commands;
pub mod frame_info;
mod host;
mod loader;
mod update;
pub use host::state::HostState;
pub use loader::*;

/// A loaded script and the per-script state the host tracks for it.
pub struct ScriptInstance {
    /// The actual instance
    pub instance: wasmtime::Instance,
    /// This script's store
    pub store: wasmtime::Store<HostState>,
    /// `Time::elapsed()` when the script was loaded.
    pub loaded_at: std::time::Duration,
    /// Frame number the next frame info write will carry, starting at 0.
    pub frame: u64,
    /// Set once a call into the script traps. A trap doesn't unwind, so the
    /// guest may be stuck mid-write (held `RefCell` borrows, a half-encoded
    /// command, a heap left mid-allocation). The host never calls into or reads
    /// from a poisoned instance again.
    pub poisoned: bool,
}

impl ScriptInstance {
    /// Marks the script dead after a trap so the host stops touching it.
    pub fn poison(&mut self, reason: &str) {
        if !self.poisoned {
            tracing::error!(
                "poisoning script instance, it will no longer run: {}",
                reason
            );
        }
        self.poisoned = true;
    }
}

#[derive(Resource)]
pub struct RuntimeSandboxResource {
    pub engine: wasmtime::Engine,
    pub instances: Vec<ScriptInstance>,
    pub linker: wasmtime::Linker<HostState>,
}

impl Plugin for RuntimeSandboxPlugin {
    fn build(&self, app: &mut App) {
        let mut config = wasmtime::Config::default();
        config.epoch_interruption(true);

        let engine = match wasmtime::Engine::new(&config) {
            Ok(v) => v,
            Err(e) => panic!("FAILED TO MAKE WASM ENGINE: {}", e),
        };
        tracing::info!("Created wasm engine");

        spawn_epoch_ticker(&engine);

        let mut linker = wasmtime::Linker::new(&engine);
        tracing::info!("Created wasm linker");

        // link logging
        match host::log::link(&mut linker) {
            Ok(()) => tracing::info!("Linked host log"),
            Err(e) => tracing::error!("FAILED to link host log with reason {}", e),
        }

        let pending_title = host::window::PendingTitle::default();
        match host::window::link(&mut linker, &pending_title) {
            Ok(()) => tracing::info!("Linked host set_title"),
            Err(e) => tracing::error!("FAILED to link host set_title with reason {}", e),
        }
        app.insert_resource(pending_title);

        match host::flush::link(&mut linker) {
            Ok(()) => tracing::info!("Linked host flush_commands"),
            Err(e) => tracing::error!("FAILED to link host flush_commands with reason {}", e),
        }

        app.insert_resource(RuntimeSandboxResource {
            engine,
            instances: Vec::new(),
            linker,
        });
        tracing::info!("Inserted resource");

        app.add_observer(observe_load_module_event);
        app.add_observer(observe_pack_read_event);

        app.add_systems(
            Update,
            (
                frame_info::write_frame_info_update,
                update::pre_physics_update,
                update::post_physics_update,
                update::pre_render_update,
            )
                .chain(),
        );
        // the previous frame has been rendered by the time `First` runs
        app.add_systems(First, update::post_render_update);
        app.add_systems(Update, host::window::apply_pending_title);
    }
}

/// increments the engine's epoch every `WASM_EPOCH_TICK` so store deadlines measure wall-clock time.
/// holds only a weak handle, so the thread exits once the engine is dropped
fn spawn_epoch_ticker(engine: &wasmtime::Engine) {
    let weak = engine.weak();
    let spawned = std::thread::Builder::new()
        .name("wasm-epoch-ticker".into())
        .spawn(move || {
            loop {
                std::thread::sleep(WASM_EPOCH_TICK);
                match weak.upgrade() {
                    Some(engine) => engine.increment_epoch(),
                    None => break,
                }
            }
        });

    match spawned {
        Ok(_) => tracing::info!("Started wasm epoch ticker ({:?})", WASM_EPOCH_TICK),
        Err(e) => panic!("FAILED TO START WASM EPOCH TICKER: {}", e),
    }
}
