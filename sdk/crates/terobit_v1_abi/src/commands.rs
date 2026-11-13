//! Layout of the command buffer the GUEST writes and the HOST reads.
//!
//! The buffer is a flat list of records, each laid out as:
//!
//! | bytes          | field                                  |
//! |----------------|----------------------------------------|
//! | `u32` (LE)     | opcode, one of the constants below     |
//! | `u32` (LE)     | payload length in bytes                |
//! | `[u8; length]` | payload, format depends on the opcode  |
//!
//! The HOST reads the buffer at the end of every cycle (through the
//! [`COMMANDS_PTR_ENDPOINT`](crate::endpoints::COMMANDS_PTR_ENDPOINT) and
//! [`COMMANDS_LEN_ENDPOINT`](crate::endpoints::COMMANDS_LEN_ENDPOINT) exports),
//! or immediately when the GUEST calls
//! [`flush_commands`](crate::crosspoints::commands::flush_commands).

pub type Opcode = u32;

/// Size of the record header (opcode + payload length).
pub const HEADER_LEN: usize = 8;

/// Spawn a new entity.
///
/// Payload: TODO.
pub const SPAWN: Opcode = 0;
