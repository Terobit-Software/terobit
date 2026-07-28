use std::collections::HashMap;

use bevy_ecs::{entity::Entity, world::World};
use glam::Mat4;

use crate::{
    ecs::{Material, Mesh, Transform},
    rendering::{InstanceRaw, MeshSourceId},
};

/// groups all (Transform, Mesh) entities by mesh id, packing each into the `InstanceRaw`
/// format the gpu wants directly (no separate model/prev_model/color vecs re-zipped
/// later), plus each entity's last-frame model matrix so the shader can compute
/// per-pixel motion vectors for taa. entities missing from `prev_transforms` (new this
/// frame) get zero velocity, since there's nothing to reproject from yet.
///
/// both `prev_transforms` and `containers` are caller-owned and reused frame to frame
/// instead of rebuilt, so steady-state frames don't pay for reallocating/rehashing a
/// ~10k-entry map and its vecs every single frame - `containers`' vecs are cleared
/// in place (retaining capacity) before this repopulates them. the tradeoff: despawned
/// entities leave a stale `prev_transforms` entry behind instead of being pruned
/// automatically. fine for now since this engine's scenes are populated once and don't
/// churn entities
pub(crate) fn accumulate_meshes(
    scene: &mut World,
    prev_transforms: &mut HashMap<Entity, Mat4>,
    containers: &mut HashMap<MeshSourceId, Vec<InstanceRaw>>,
) {
    for instances in containers.values_mut() {
        instances.clear();
    }

    for (entity, transform, mesh, material) in scene
        .query::<(Entity, &Transform, &Mesh, Option<&Material>)>()
        .iter(scene)
    {
        let color = material.map(|m| m.color).unwrap_or([1.0, 1.0, 1.0, 1.0]);
        let source_id = *mesh.get_id();
        let model = transform.to_mat4();
        // insert() returns the old value in one lookup instead of a separate get() + insert()
        let prev_model = prev_transforms.insert(entity, model).unwrap_or(model);

        containers
            .entry(source_id)
            .or_default()
            .push(InstanceRaw::new(model, prev_model, color));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ecs::{Mesh, Transform},
        rendering::MeshSourceId,
    };
    use glam::Vec3;

    fn run(
        world: &mut World,
        prev_transforms: &mut HashMap<Entity, Mat4>,
    ) -> HashMap<MeshSourceId, Vec<InstanceRaw>> {
        let mut containers = HashMap::new();
        accumulate_meshes(world, prev_transforms, &mut containers);
        containers
    }

    #[test]
    fn empty_world_gives_empty_map() {
        let mut world = World::new();
        let containers = run(&mut world, &mut HashMap::new());
        assert!(containers.values().all(|v| v.is_empty()));
    }

    #[test]
    fn single_entity_single_container_single_transform() {
        let mut world = World::new();
        world.spawn((Transform::new(), Mesh::cube()));
        let containers = run(&mut world, &mut HashMap::new());
        assert_eq!(containers[&MeshSourceId::CUBE].len(), 1);
    }

    #[test]
    fn two_same_mesh_one_container_two_transforms() {
        let mut world = World::new();
        world.spawn((Transform::new(), Mesh::cube()));
        world.spawn((Transform::new().with_pos(Vec3::X), Mesh::cube()));
        let containers = run(&mut world, &mut HashMap::new());
        assert_eq!(containers[&MeshSourceId::CUBE].len(), 2);
    }

    #[test]
    fn transform_without_mesh_not_included() {
        let mut world = World::new();
        world.spawn(Transform::new());
        let containers = run(&mut world, &mut HashMap::new());
        assert!(containers.values().all(|v| v.is_empty()));
    }

    #[test]
    fn material_color_stored_per_instance() {
        let mut world = World::new();
        world.spawn((Transform::new(), Mesh::cube(), Material::red()));
        world.spawn((Transform::new(), Mesh::cube(), Material::blue()));
        let containers = run(&mut world, &mut HashMap::new());
        let instances = &containers[&MeshSourceId::CUBE];
        assert_eq!(instances.len(), 2);
        // both colors present (order not guaranteed)
        let has_red = instances.iter().any(|i| i.color() == [1.0, 0.0, 0.0, 1.0]);
        let has_blue = instances.iter().any(|i| i.color() == [0.0, 0.0, 1.0, 1.0]);
        assert!(has_red && has_blue);
    }

    #[test]
    fn no_material_defaults_to_white() {
        let mut world = World::new();
        world.spawn((Transform::new(), Mesh::cube()));
        let containers = run(&mut world, &mut HashMap::new());
        assert_eq!(containers[&MeshSourceId::CUBE][0].color(), [1.0, 1.0, 1.0, 1.0]);
    }

    #[test]
    fn new_entity_has_zero_velocity_prev_equals_current() {
        let mut world = World::new();
        world.spawn((
            Transform::new().with_pos(Vec3::new(5.0, 0.0, 0.0)),
            Mesh::cube(),
        ));
        let containers = run(&mut world, &mut HashMap::new());
        let instance = &containers[&MeshSourceId::CUBE][0];
        assert_eq!(instance.model(), instance.prev_model());
    }

    #[test]
    fn prev_transforms_updated_in_place_feed_next_frame() {
        let mut world = World::new();
        let entity = world.spawn((Transform::new(), Mesh::cube())).id();
        let mut history = HashMap::new();
        run(&mut world, &mut history);

        {
            let mut transform = world.get_mut::<Transform>(entity).unwrap();
            transform.position = Vec3::new(10.0, 0.0, 0.0);
        }

        let containers = run(&mut world, &mut history);
        let instance = &containers[&MeshSourceId::CUBE][0];
        assert_ne!(instance.model(), instance.prev_model());
    }

    #[test]
    fn vecs_cleared_not_reallocated_between_calls() {
        let mut world = World::new();
        world.spawn((Transform::new(), Mesh::cube()));
        world.spawn((Transform::new(), Mesh::cube()));
        let mut containers = HashMap::new();
        let mut prev_transforms = HashMap::new();
        accumulate_meshes(&mut world, &mut prev_transforms, &mut containers);
        let capacity_after_first = containers[&MeshSourceId::CUBE].capacity();

        // second frame, one fewer entity - the vec should shrink in length but keep
        // its allocation rather than being replaced
        world.clear_entities();
        world.spawn((Transform::new(), Mesh::cube()));
        accumulate_meshes(&mut world, &mut prev_transforms, &mut containers);

        assert_eq!(containers[&MeshSourceId::CUBE].len(), 1);
        assert!(containers[&MeshSourceId::CUBE].capacity() >= capacity_after_first);
    }
}
