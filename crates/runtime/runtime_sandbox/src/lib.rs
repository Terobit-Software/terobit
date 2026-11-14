use bevy::{
    ecs::resource::Resource,
    prelude::{App, First, IntoScheduleConfigs, Plugin, Update},
};
use runtime_consts::sandbox::WASM_FUEL;

pub struct RuntimeSandboxPlugin;

mod host;
mod loader;
mod update;
pub use loader::*;

#[derive(Resource)]
pub struct RuntimeSandboxResource {
    pub engine: wasmtime::Engine,
    pub store: wasmtime::Store<()>,
    pub instances: Vec<wasmtime::Instance>,
    pub linker: wasmtime::Linker<()>,
}

impl Plugin for RuntimeSandboxPlugin {
    fn build(&self, app: &mut App) {
        let mut config = wasmtime::Config::default();
        config.consume_fuel(true);

        let engine = match wasmtime::Engine::new(&config) {
            Ok(v) => v,
            Err(e) => panic!("FAILED TO MAKE WASM ENGINE: {}", e),
        };
        tracing::info!("Created wasm engine");

        let mut store = wasmtime::Store::new(&engine, ());
        tracing::info!("Created wasm store");

        match store.set_fuel(WASM_FUEL) {
            Ok(()) => tracing::info!("Allocated {} fuel successfully", WASM_FUEL),
            Err(e) => tracing::error!("FAILED to allocate {} fuel with reason {}", WASM_FUEL, e),
        }

        let mut linker = wasmtime::Linker::new(&engine);
        tracing::info!("Created wasm linker");

        match host::log::link(&mut linker) {
            Ok(()) => tracing::info!("Linked host log"),
            Err(e) => tracing::error!("FAILED to link host log with reason {}", e),
        }

        app.insert_resource(RuntimeSandboxResource {
            engine,
            store,
            instances: Vec::new(),
            linker,
        });
        tracing::info!("Inserted resource");

        app.add_observer(observe_load_module_event);
        app.add_observer(observe_pack_read_event);

        app.add_systems(
            Update,
            (
                update::pre_physics_update,
                update::post_physics_update,
                update::pre_render_update,
            )
                .chain(),
        );
        // the previous frame has been rendered by the time `First` runs
        app.add_systems(First, update::post_render_update);
    }
}
