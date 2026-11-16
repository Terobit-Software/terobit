use terobit_v1_abi::commands::{Opcode, SPAWN};

use crate::ecs::Command;

/// Spawns a new entity.
pub struct Spawn {
    // TODO: components to spawn the entity with.
}

impl Spawn {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for Spawn {
    fn default() -> Self {
        Self::new()
    }
}

impl Command for Spawn {
    const OPCODE: Opcode = SPAWN;

    fn encode(&self, _payload: &mut Vec<u8>) {
        // TODO: encode components.
    }
}
