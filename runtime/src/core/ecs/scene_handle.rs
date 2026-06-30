use bevy_ecs::prelude::*;
use bevy_ecs::query::QueryData;
use bevy_ecs::world::World;

/// a thin wrapper around a bevy_ecs `World`, used to interact with a scene
pub struct SceneHandle<'a> {
    pub scene: &'a mut World,
}

impl SceneHandle<'_> {
    /// spawns an entity with the given bundle and returns its id
    pub fn spawn<B: Bundle>(&mut self, bundle: B) -> Entity {
        self.scene.spawn(bundle).id()
    }

    /// creates a query against the scene's entities
    pub fn query<D: QueryData> (&mut self) -> QueryState<D> {
        self.scene.query::<D>()
    }

    /// returns a shared reference to the underlying world
    pub fn raw (&self) -> &World {
        &self.scene
    }

    /// returns a mutable reference to the underlying world
    pub fn mut_raw (&mut self) -> &mut World {
        &mut self.scene
    }
}
