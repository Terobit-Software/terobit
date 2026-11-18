use terobit_v1_abi::commands::{Opcode, SPAWN};

use crate::ecs::{
    Command,
    components::{Component, ComponentId},
};

/// Spawns a new entity.
#[derive(Default)]
pub struct Spawn {
    // id, raw bytes
    components: Vec<(ComponentId, Vec<u8>)>,
}

impl Spawn {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with<C: Component + bytemuck::Pod>(mut self, component: C) -> Self {
        self.components
            .push((C::ID, bytemuck::bytes_of(&component).to_vec()));
        self
    }
}

impl Command for Spawn {
    const OPCODE: Opcode = SPAWN;

    fn encode(&self, _payload: &mut Vec<u8>) {
        // TODO: encode components.
    }
}
