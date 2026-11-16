use core::cell::RefCell;

use terobit_v1_abi::commands::HEADER_LEN;

use crate::ecs::{Command, Spawn};

thread_local! {
    /// The guest's command buffer. The host reads it in place.
    static BUFFER: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

/// Queues [`Command`]s into the command buffer.
///
/// Nothing is sent to the host here; the host reads the buffer at the end of
/// the cycle, or when [`flush`](Self::flush) is called.
pub struct Commands;

impl Commands {
    /// Writes a command into the buffer.
    pub fn push<C: Command>(command: C) {
        BUFFER.with_borrow_mut(|buffer| {
            let header = buffer.len();
            buffer.extend_from_slice(&C::OPCODE.to_le_bytes());
            buffer.extend_from_slice(&0u32.to_le_bytes());

            command.encode(buffer);

            let payload_len = (buffer.len() - header - HEADER_LEN) as u32;
            buffer[header + 4..header + HEADER_LEN].copy_from_slice(&payload_len.to_le_bytes());
        });
    }

    /// Queues a [`Spawn`].
    pub fn spawn() {
        Self::push(Spawn::new());
    }

    /// Asks the host to apply everything queued so far, then clears the buffer.
    pub fn flush() {
        BUFFER.with_borrow_mut(|buffer| {
            if buffer.is_empty() {
                return;
            }
            // SAFETY: the host only reads `len` bytes from `ptr` during the call,
            // and wasm32 pointers and lengths always fit in an i32.
            unsafe {
                terobit_v1_abi::crosspoints::commands::flush_commands(
                    buffer.as_ptr() as i32,
                    buffer.len() as i32,
                );
            }
            buffer.clear();
        });
    }
}

// Exports the host uses to read the buffer at the end of every cycle.
// Names match `terobit_v1_abi::endpoints::COMMANDS_*_ENDPOINT`.

#[unsafe(no_mangle)]
pub extern "C" fn _terobit_v1_commands_ptr() -> i32 {
    BUFFER.with_borrow(|buffer| buffer.as_ptr() as i32)
}

#[unsafe(no_mangle)]
pub extern "C" fn _terobit_v1_commands_len() -> i32 {
    BUFFER.with_borrow(|buffer| buffer.len() as i32)
}

#[unsafe(no_mangle)]
pub extern "C" fn _terobit_v1_commands_clear() {
    BUFFER.with_borrow_mut(|buffer| buffer.clear());
}
