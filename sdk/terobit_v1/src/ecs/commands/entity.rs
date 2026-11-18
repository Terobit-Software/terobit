use terobit_v1_abi::commands::{Opcode, SPAWN};

use crate::ecs::Command;

/// Spawns a new entity.
#[derive(Default)]
pub struct Spawn;

impl Spawn {
    pub fn new() -> Self {
        Self
    }
}

impl Command for Spawn {
    const OPCODE: Opcode = SPAWN;

    fn encode(&self, _payload: &mut Vec<u8>) {}
}
