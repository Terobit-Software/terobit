use bevy::ecs::system::ResMut;
use runtime_consts::sandbox::WASM_CYCLE_TICKS;
use terobit_v1_abi::endpoints;

use crate::RuntimeSandboxResource;

/// calls `name` on every instance that exports it, under a fresh epoch deadline
/// so every cycle (pre-physics, post-physics, pre-render, post-render) gets a full time budget
fn call_entrypoint(sandbox: &mut RuntimeSandboxResource, name: &str) {
    let RuntimeSandboxResource {
        store, instances, ..
    } = sandbox;

    store.set_epoch_deadline(WASM_CYCLE_TICKS);

    for instance in instances.iter() {
        let Ok(func) = instance.get_typed_func::<(), ()>(&mut *store, name) else {
            continue;
        };
        if let Err(e) = func.call(&mut *store, ()) {
            tracing::error!("`{}` failed: {:?}", name, e);
        }
    }
}

pub fn pre_physics_update(mut sandbox: ResMut<RuntimeSandboxResource>) {
    call_entrypoint(&mut sandbox, endpoints::PRE_PHYSICS_ENTRYPOINT);
}

pub fn post_physics_update(mut sandbox: ResMut<RuntimeSandboxResource>) {
    call_entrypoint(&mut sandbox, endpoints::POST_PHYSICS_ENTRYPOINT);
}

pub fn pre_render_update(mut sandbox: ResMut<RuntimeSandboxResource>) {
    call_entrypoint(&mut sandbox, endpoints::PRE_RENDER_ENTRYPOINT);
}

pub fn post_render_update(mut sandbox: ResMut<RuntimeSandboxResource>) {
    call_entrypoint(&mut sandbox, endpoints::POST_RENDER_ENTRYPOINT);
}
