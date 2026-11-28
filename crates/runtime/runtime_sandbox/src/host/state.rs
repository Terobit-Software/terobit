use std::ptr::NonNull;

use bevy::ecs::world::World;
use runtime_consts::sandbox::WASM_MEMORY_LIMIT;
use wasmtime::{StoreLimits, StoreLimitsBuilder};

pub struct HostState {
    pub(crate) world: Option<NonNull<World>>,
    /// caps how far this script's memory can grow, see [`WASM_MEMORY_LIMIT`]
    pub(crate) limits: StoreLimits,
}

impl HostState {
    pub(crate) fn new() -> Self {
        Self {
            world: None,
            limits: StoreLimitsBuilder::new()
                .memory_size(WASM_MEMORY_LIMIT)
                .build(),
        }
    }
}

unsafe impl Send for HostState {}
unsafe impl Sync for HostState {}

pub(crate) fn with_world<R>(
    store: &mut wasmtime::Store<HostState>,
    world: &mut World,
    f: impl FnOnce(&mut wasmtime::Store<HostState>) -> R,
) -> R {
    store.data_mut().world = Some(NonNull::from(world));
    let result = f(store);
    store.data_mut().world = None;
    result
}
