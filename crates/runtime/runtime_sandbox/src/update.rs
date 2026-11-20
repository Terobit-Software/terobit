use bevy::ecs::system::{Commands, ResMut};
use runtime_consts::sandbox::WASM_CYCLE_TICKS;
use terobit_v1_abi::endpoints;

use crate::{RuntimeSandboxResource, commands::read_and_apply_commands};

/// calls `name` on every instance that exports it, under a fresh epoch deadline
/// so every cycle (pre-physics, post-physics, pre-render, post-render) gets a full time budget.
/// each instance's command buffer is read and applied at the end of its cycle.
/// an instance that traps is poisoned: its commands from that cycle are dropped
/// and it is skipped from then on
fn call_entrypoint(sandbox: &mut RuntimeSandboxResource, commands: &mut Commands, name: &str) {
    let RuntimeSandboxResource {
        store, instances, ..
    } = sandbox;

    store.set_epoch_deadline(WASM_CYCLE_TICKS);

    for script in instances.iter_mut().filter(|s| !s.poisoned) {
        let instance = script.instance;
        let Ok(func) = instance.get_typed_func::<(), ()>(&mut *store, name) else {
            continue;
        };
        if let Err(e) = func.call(&mut *store, ()) {
            tracing::error!("`{}` failed: {:?}", name, e);
            script.poison(&format!("`{}` trapped", name));
            continue;
        }
        read_and_apply_commands(&instance, store, commands);
    }
}

pub fn pre_physics_update(mut sandbox: ResMut<RuntimeSandboxResource>, mut commands: Commands) {
    call_entrypoint(
        &mut sandbox,
        &mut commands,
        endpoints::PRE_PHYSICS_ENTRYPOINT,
    );
}

pub fn post_physics_update(mut sandbox: ResMut<RuntimeSandboxResource>, mut commands: Commands) {
    call_entrypoint(
        &mut sandbox,
        &mut commands,
        endpoints::POST_PHYSICS_ENTRYPOINT,
    );
}

pub fn pre_render_update(mut sandbox: ResMut<RuntimeSandboxResource>, mut commands: Commands) {
    call_entrypoint(
        &mut sandbox,
        &mut commands,
        endpoints::PRE_RENDER_ENTRYPOINT,
    );
}

pub fn post_render_update(mut sandbox: ResMut<RuntimeSandboxResource>, mut commands: Commands) {
    call_entrypoint(
        &mut sandbox,
        &mut commands,
        endpoints::POST_RENDER_ENTRYPOINT,
    );
}
