//! Host side of the guest frame info buffer (see `terobit_v1_abi::frame_info`).

use crate::HostState;
use bevy::{
    ecs::system::{Res, ResMut},
    time::Time,
};
use snafu::{OptionExt, ResultExt, Snafu};
use terobit_v1_abi::{endpoints, frame_info::FrameInfo};

use crate::{RuntimeSandboxResource, ScriptInstance};

#[derive(Debug, Snafu)]
pub enum WriteFrameInfoError {
    #[snafu(display(
        "guest is missing frame info export `{}`",
        endpoints::FRAME_INFO_PTR_ENDPOINT
    ))]
    MissingExport { source: wasmtime::Error },

    #[snafu(display(
        "guest frame info export `{}` failed",
        endpoints::FRAME_INFO_PTR_ENDPOINT
    ))]
    ExportFailed { source: wasmtime::Error },

    #[snafu(display("guest has no exported `memory`"))]
    NoMemory,

    #[snafu(display("frame info buffer at {ptr} is out of guest memory bounds"))]
    OutOfBounds {
        ptr: usize,
        source: wasmtime::MemoryAccessError,
    },
}

/// Writes `info` into `instance`'s frame info buffer.
pub fn write_frame_info(
    instance: &wasmtime::Instance,
    store: &mut wasmtime::Store<HostState>,
    info: &FrameInfo,
) -> Result<(), WriteFrameInfoError> {
    let ptr = instance
        .get_typed_func::<(), i32>(&mut *store, endpoints::FRAME_INFO_PTR_ENDPOINT)
        .context(MissingExportSnafu)?
        .call(&mut *store, ())
        .context(ExportFailedSnafu)? as u32 as usize;

    let memory = instance
        .get_memory(&mut *store, "memory")
        .context(NoMemorySnafu)?;

    memory
        .write(&mut *store, ptr, bytemuck::bytes_of(info))
        .context(OutOfBoundsSnafu { ptr })
}

/// Writes the current frame's [`FrameInfo`] into every instance, then advances
/// its frame counter. Runs once per frame, before the first update stage.
pub fn write_frame_info_update(mut sandbox: ResMut<RuntimeSandboxResource>, time: Res<Time>) {
    let RuntimeSandboxResource {
        store, instances, ..
    } = &mut *sandbox;

    for script in instances.iter_mut().filter(|s| !s.poisoned) {
        let ScriptInstance {
            instance,
            loaded_at,
            frame,
            ..
        } = script;

        let info = FrameInfo {
            delta_secs: time.delta_secs_f64(),
            elapsed_secs: time.elapsed().saturating_sub(*loaded_at).as_secs_f64(),
            frame: *frame,
        };
        *frame += 1;

        match write_frame_info(instance, store, &info) {
            Ok(()) => {}
            Err(WriteFrameInfoError::MissingExport { .. }) => {
                tracing::trace!(
                    "guest has no `{}`, skipping frame info",
                    endpoints::FRAME_INFO_PTR_ENDPOINT
                );
            }
            Err(e) => tracing::error!(
                "Failed to write guest frame info: {}",
                snafu::Report::from_error(&e)
            ),
        }
    }
}
