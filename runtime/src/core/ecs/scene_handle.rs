use bevy_ecs::prelude::*;
use bevy_ecs::query::QueryData;
use bevy_ecs::world::World;

pub struct SceneHandle<'a> {
    pub scene: &'a mut World,
}

impl SceneHandle<'_> {
    pub fn spawn<B>(&mut self, bundle: B) -> Entity
    where
        B: Bundle,
    {
        self.scene.spawn(bundle).id()
    }

    pub fn query<D: QueryData> (&mut self) -> QueryState<D> {
        self.scene.query::<D>()
    }

    pub fn raw (&self) -> &World {
        self.scene
    }
}
