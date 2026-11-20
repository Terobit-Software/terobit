use bevy::ecs::world::{Mut, World};
use runtime_consts::sandbox::WASM_CYCLE_TICKS;
use terobit_v1_abi::endpoints;

use crate::{RuntimeSandboxResource, commands::read_and_apply_commands, host::state::with_world};

/// calls `name` on every instance that exports it, under a fresh epoch deadline
/// so every cycle (pre-physics, post-physics, pre-render, post-render) gets a full time budget.
/// each instance's command buffer is read and applied at the end of its cycle.
/// an instance that traps is poisoned: its commands from that cycle are dropped
/// and it is skipped from then on
fn call_entrypoint(world: &mut World, name: &str) {
    world.resource_scope(|world, mut sandbox: Mut<RuntimeSandboxResource>| {
        let RuntimeSandboxResource {
            store, instances, ..
        } = &mut *sandbox;

        store.set_epoch_deadline(WASM_CYCLE_TICKS);

        for script in instances.iter_mut().filter(|s| !s.poisoned) {
            let instance = script.instance;
            let Ok(func) = instance.get_typed_func::<(), ()>(&mut *store, name) else {
                continue;
            };
            if let Err(e) = with_world(store, world, |store| func.call(store, ())) {
                tracing::error!("`{}` failed: {:?}", name, e);
                script.poison(&format!("`{}` trapped", name));
                continue;
            }
            read_and_apply_commands(&instance, store, world);
        }
    });
}

pub fn pre_physics_update(world: &mut World) {
    call_entrypoint(world, endpoints::PRE_PHYSICS_ENTRYPOINT);
}

pub fn post_physics_update(world: &mut World) {
    call_entrypoint(world, endpoints::POST_PHYSICS_ENTRYPOINT);
}

pub fn pre_render_update(world: &mut World) {
    call_entrypoint(world, endpoints::PRE_RENDER_ENTRYPOINT);
}

pub fn post_render_update(world: &mut World) {
    call_entrypoint(world, endpoints::POST_RENDER_ENTRYPOINT);
}
