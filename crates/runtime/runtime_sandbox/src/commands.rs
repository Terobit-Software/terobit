//! Host side of the guest command buffer (see `terobit_v1_abi::commands`).

use snafu::{OptionExt, ResultExt, Snafu};
use terobit_v1_abi::{
    commands::{HEADER_LEN, Opcode},
    endpoints,
};

/// One record read out of a guest's command buffer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuestCommand {
    pub opcode: Opcode,
    pub payload: Vec<u8>,
}

#[derive(Debug, Snafu)]
pub enum ReadCommandsError {
    #[snafu(display("guest is missing command buffer export `{}`", name))]
    MissingExport {
        name: &'static str,
        source: wasmtime::Error,
    },

    #[snafu(display("guest command buffer export `{}` failed", name))]
    ExportFailed {
        name: &'static str,
        source: wasmtime::Error,
    },

    #[snafu(display("guest has no exported `memory`"))]
    NoMemory,

    #[snafu(display("command buffer {start}..{end} is out of guest memory bounds"))]
    OutOfBounds { start: usize, end: usize },

    #[snafu(display("malformed command record at byte {offset}"))]
    Malformed { offset: usize },
}

/// Reads `instance`'s command buffer, parses it into records, then clears it
/// so the same commands are not read again next cycle.
///
/// The buffer is only cleared once it has been read and parsed successfully.
pub fn read_commands(
    instance: &wasmtime::Instance,
    store: &mut wasmtime::Store<()>,
) -> Result<Vec<GuestCommand>, ReadCommandsError> {
    let ptr = call_i32(instance, store, endpoints::COMMANDS_PTR_ENDPOINT)?;
    let len = call_i32(instance, store, endpoints::COMMANDS_LEN_ENDPOINT)?;

    if len == 0 {
        return Ok(Vec::new());
    }

    let memory = instance
        .get_memory(&mut *store, "memory")
        .context(NoMemorySnafu)?;

    let commands = read_commands_from_memory(memory.data(&*store), ptr, len)?;

    let clear = instance
        .get_typed_func::<(), ()>(&mut *store, endpoints::COMMANDS_CLEAR_ENDPOINT)
        .context(MissingExportSnafu {
            name: endpoints::COMMANDS_CLEAR_ENDPOINT,
        })?;
    clear.call(&mut *store, ()).context(ExportFailedSnafu {
        name: endpoints::COMMANDS_CLEAR_ENDPOINT,
    })?;

    Ok(commands)
}

/// Parses the command buffer at `ptr..ptr + len` in guest `memory`.
///
/// Shared by [`read_commands`] and the `flush_commands` crosspoint, which is
/// handed `ptr`/`len` directly.
pub fn read_commands_from_memory(
    memory: &[u8],
    ptr: i32,
    len: i32,
) -> Result<Vec<GuestCommand>, ReadCommandsError> {
    let start = ptr as u32 as usize;
    let end = start + len as u32 as usize;
    let buffer = memory
        .get(start..end)
        .context(OutOfBoundsSnafu { start, end })?;

    parse_commands(buffer)
}

/// Splits a raw command buffer into its records.
pub fn parse_commands(mut buffer: &[u8]) -> Result<Vec<GuestCommand>, ReadCommandsError> {
    let total = buffer.len();
    let mut commands = Vec::new();

    while !buffer.is_empty() {
        let offset = total - buffer.len();

        let (header, rest) = buffer
            .split_first_chunk::<HEADER_LEN>()
            .context(MalformedSnafu { offset })?;
        let opcode = u32::from_le_bytes([header[0], header[1], header[2], header[3]]);
        let payload_len = u32::from_le_bytes([header[4], header[5], header[6], header[7]]) as usize;

        let (payload, rest) = rest
            .split_at_checked(payload_len)
            .context(MalformedSnafu { offset })?;

        commands.push(GuestCommand {
            opcode,
            payload: payload.to_vec(),
        });
        buffer = rest;
    }

    Ok(commands)
}

fn call_i32(
    instance: &wasmtime::Instance,
    store: &mut wasmtime::Store<()>,
    name: &'static str,
) -> Result<i32, ReadCommandsError> {
    instance
        .get_typed_func::<(), i32>(&mut *store, name)
        .context(MissingExportSnafu { name })?
        .call(&mut *store, ())
        .context(ExportFailedSnafu { name })
}
