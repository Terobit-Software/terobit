use terobit_v1_abi::commands::{Opcode, SPAWN};

use crate::ecs::{Command, components::ComponentId};

/// Spawns a new entity.
#[derive(Default)]
pub struct Spawn {
    // id, raw bytes
    components: Vec<(ComponentId, Vec<u8>)>,
}

impl Spawn {
    pub fn new(components: Vec<(ComponentId, Vec<u8>)>) -> Self {
        Self { components }
    }
}

impl Command for Spawn {
    const OPCODE: Opcode = SPAWN;

    /// Payload: for each component, `id: u32 LE`, `len: u32 LE`, then `len`
    /// bytes.
    fn encode(&self, payload: &mut Vec<u8>) {
        for (id, bytes) in &self.components {
            payload.extend_from_slice(&id.to_le_bytes()); // the component id
            payload.extend_from_slice(&(bytes.len() as u32).to_le_bytes()); // the payload length
            payload.extend_from_slice(bytes); // the payload
        }
    }
}
