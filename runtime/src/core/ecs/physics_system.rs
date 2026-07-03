use std::collections::HashMap;

use bevy_ecs::prelude::Entity;
use bevy_ecs::world::World;
use glam::Vec3;
use rapier3d::prelude::{ColliderBuilder, Pose, PhysicsWorld, RigidBodyBuilder};

use crate::{
    consts::{GRAVITY, MAX_PHYSICS_DT},
    ecs::{Collider, Mesh, RigidBody, Transform},
    rendering::MeshSourceId,
};

/// exact-bits scale key so identical scales (the common case: most spawned entities
/// share a handful of scales) reuse the same collider shape instead of rebuilding a
/// convex hull per entity
type ScaleKey = (u32, u32, u32);

fn scale_key(scale: Vec3) -> ScaleKey {
    (scale.x.to_bits(), scale.y.to_bits(), scale.z.to_bits())
}

/// owns the rapier simulation for one scene. long-lived: created once and stepped
/// every frame. entities are registered into it lazily, the first physics step that
/// sees a `RigidBody` without a cached handle inserts it
pub(crate) struct PhysicsScene {
    world: PhysicsWorld,
    /// cache of built collider shapes keyed by (mesh id, scale), so entities that
    /// share a mesh and scale (e.g. a few thousand identical cubes) don't each pay
    /// for their own convex-hull computation
    shape_cache: HashMap<(MeshSourceId, ScaleKey), ColliderBuilder>,
}

impl PhysicsScene {
    pub fn new() -> Self {
        let mut world = PhysicsWorld::new();
        world.gravity = GRAVITY;
        Self { world, shape_cache: HashMap::new() }
    }

    /// registers any not-yet-registered rigid bodies, steps the simulation, then
    /// writes the resulting transforms back onto the ecs. dt is clamped so a stalled
    /// frame (window drag, resize) doesn't blow up the integration
    pub fn step(&mut self, scene: &mut World, dt: f32, mesh_points: &HashMap<MeshSourceId, Vec<Vec3>>) {
        let dt = dt.min(MAX_PHYSICS_DT);
        if dt <= 0.0 {
            return;
        }

        self.register_new_bodies(scene, mesh_points);

        self.world.integration_parameters.dt = dt;
        self.world.step();

        self.sync_transforms(scene);
    }

    fn register_new_bodies(&mut self, scene: &mut World, mesh_points: &HashMap<MeshSourceId, Vec<Vec3>>) {
        let mut query = scene.query::<(Entity, &Transform, &RigidBody, &Collider, Option<&Mesh>)>();

        let pending: Vec<_> = query
            .iter(scene)
            .filter(|(_, _, body, ..)| body.handle.is_none())
            .map(|(entity, transform, body, collider, mesh)| {
                (entity, *transform, body.mass, body.restitution, body.is_static, collider.clone(), mesh.map(|m| *m.get_id()))
            })
            .collect();

        for (entity, transform, mass, restitution, is_static, collider, mesh_id) in pending {
            let Some(shape) = self.shape_for(&collider, mesh_id, transform.scale, mesh_points) else {
                continue;
            };

            let body_builder = if is_static {
                RigidBodyBuilder::fixed()
            } else {
                RigidBodyBuilder::dynamic().additional_mass(mass.max(0.001))
            }
            .position(Pose::from_parts(transform.position, transform.rotation));

            let collider_builder = shape.restitution(restitution).friction(0.6);

            let (handle, _) = self.world.insert(body_builder, collider_builder);

            if let Some(mut body) = scene.get_mut::<RigidBody>(entity) {
                body.handle = Some(handle);
            }
        }
    }

    /// builds (or reuses a cached) collider shape for the given collider descriptor
    fn shape_for(
        &mut self,
        collider: &Collider,
        mesh_id: Option<MeshSourceId>,
        scale: Vec3,
        mesh_points: &HashMap<MeshSourceId, Vec<Vec3>>,
    ) -> Option<ColliderBuilder> {
        match collider {
            Collider::Box { half_extents } => {
                let e = *half_extents * scale.abs();
                Some(ColliderBuilder::cuboid(e.x.max(0.001), e.y.max(0.001), e.z.max(0.001)))
            }
            Collider::Inherit => {
                let mesh_id = mesh_id?;
                let key = (mesh_id, scale_key(scale));

                if let Some(cached) = self.shape_cache.get(&key) {
                    return Some(cached.clone());
                }

                let points = mesh_points.get(&mesh_id)?;
                let scaled: Vec<Vec3> = points.iter().map(|p| *p * scale).collect();
                let builder = ColliderBuilder::convex_hull(&scaled)?;

                self.shape_cache.insert(key, builder.clone());
                Some(builder)
            }
        }
    }

    fn sync_transforms(&self, scene: &mut World) {
        let mut query = scene.query::<(&mut Transform, &RigidBody)>();
        query.par_iter_mut(scene).for_each(|(mut transform, body)| {
            if body.is_static {
                return;
            }
            let Some(handle) = body.handle else { return };
            let Some(rb) = self.world.bodies.get(handle) else { return };
            let pose = rb.position();
            transform.position = pose.translation;
            transform.rotation = pose.rotation;
        });
    }
}
