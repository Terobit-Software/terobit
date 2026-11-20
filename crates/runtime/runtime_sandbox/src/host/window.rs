use crate::HostState;
use std::sync::{Arc, Mutex};

use bevy::{
    ecs::{query::With, resource::Resource, system::Query},
    prelude::Res,
    window::{PrimaryWindow, Window},
};
use wasmtime::{Caller, Linker, Result, bail};

/// The last title a guest asked for, waiting to be applied to the primary window.
///
/// The store has no access to the world, so `host::set_title` writes here and
/// [`apply_pending_title`] copies it into the window.
#[derive(Resource, Clone, Default)]
pub(crate) struct PendingTitle(Arc<Mutex<Option<String>>>);

/// links `host::set_title` (see `terobit_v1_abi::crosspoints::window`).
pub(crate) fn link(linker: &mut Linker<HostState>, pending: &PendingTitle) -> Result<()> {
    let pending = pending.clone();
    linker.func_wrap(
        "host",
        "set_title",
        move |caller: Caller<'_, HostState>, ptr: i32, len: i32| set_title(caller, &pending, ptr, len),
    )?;
    tracing::info!("Linked host::set_title");
    Ok(())
}

fn set_title(mut caller: Caller<'_, HostState>, pending: &PendingTitle, ptr: i32, len: i32) -> Result<()> {
    let Some(memory) = caller.get_export("memory").and_then(|e| e.into_memory()) else {
        bail!("guest has no exported `memory`");
    };

    let start = ptr as u32 as usize;
    let end = start + len as u32 as usize;
    let Some(bytes) = memory.data(&caller).get(start..end) else {
        bail!("window title {start}..{end} is out of guest memory bounds");
    };

    let title = String::from_utf8_lossy(bytes).into_owned();
    *pending.0.lock().unwrap_or_else(|e| e.into_inner()) = Some(title);
    Ok(())
}

/// Applies a title set by a guest to the primary window.
pub(crate) fn apply_pending_title(
    pending: Res<PendingTitle>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    let Some(title) = pending.0.lock().unwrap_or_else(|e| e.into_inner()).take() else {
        return;
    };
    match windows.single_mut() {
        Ok(mut window) => window.title = title,
        Err(e) => tracing::warn!(
            "guest set the window title but there is no primary window: {}",
            e
        ),
    }
}
