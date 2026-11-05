use bevy::{
    ecs::resource::Resource,
    prelude::{App, Plugin},
};

pub struct RuntimeSandboxPlugin;

mod loader;
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
        let config = wasmtime::Config::default();
        let engine = match wasmtime::Engine::new(&config) {
            Ok(v) => v,
            Err(e) => panic!("FAILED TO MAKE WASM ENGINE: {}", e),
        };
        tracing::info!("Created wasm engine");

        let store = wasmtime::Store::new(&engine, ());
        tracing::info!("Created wasm store");

        let linker = wasmtime::Linker::new(&engine);
        tracing::info!("Created wasm linker");

        app.insert_resource(RuntimeSandboxResource {
            engine,
            store,
            instances: Vec::new(),
            linker,
        });
        tracing::info!("Inserted resource");

        app.add_observer(observe_load_module_event);
    }
}
