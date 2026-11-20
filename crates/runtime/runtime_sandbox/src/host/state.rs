use std::ptr::NonNull;

use bevy::ecs::world::World;

#[derive(Default)]
pub struct HostState {
    pub(crate) world: Option<NonNull<World>>,
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
