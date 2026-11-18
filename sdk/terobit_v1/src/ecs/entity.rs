use crate::ecs::{
    Commands,
    components::{Component, ComponentId},
};

/// Builds an entity. It is spawned when the builder is dropped.
#[derive(Default)]
pub struct Entity {
    // id, raw bytes
    components: Vec<(ComponentId, Vec<u8>)>,
}

impl Entity {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with<C: Component + bytemuck::Pod>(mut self, component: C) -> Self {
        self.components
            .push((C::ID, bytemuck::bytes_of(&component).to_vec()));
        self
    }
}

impl Drop for Entity {
    fn drop(&mut self) {
        Commands::spawn(std::mem::take(&mut self.components));
    }
}
