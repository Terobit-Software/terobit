use crate::ecs::{
    Commands,
    components::{Component, ComponentId},
};

/// Builds an entity. Call [`spawn`](Self::spawn) to queue it.
#[derive(Default)]
#[must_use = "an entity is only created when `.spawn()` is called"]
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

    /// Queues the spawn. The host creates the entity when it next reads
    /// commands.
    pub fn spawn(self) {
        Commands::spawn(self.components);
    }
}
