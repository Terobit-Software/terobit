use std::{collections::HashMap, fmt::Debug};

use bevy_ecs::{entity::Entity, world::World};
use glam::Mat4;

use crate::{
    ecs::{Material, Mesh, Transform},
    rendering::MeshSourceId,
};

pub(crate) struct RenderMeshContainer {
    pub id: MeshSourceId,
    /// models, prev_models, and colors are parallel, one entry per instance
    pub models: Vec<Mat4>,
    pub prev_models: Vec<Mat4>,
    pub colors: Vec<[f32; 4]>,
}

/// groups all (Transform, Mesh) entities by mesh id, collecting per-instance colors from
/// optional Material components, plus each entity's last-frame model matrix so the
/// shader can compute per-pixel motion vectors for taa. entities missing from
/// `prev_transforms` (new this frame) get zero velocity, since there's nothing to
/// reproject from yet.
///
/// `prev_transforms` is updated in place (read this entity's old value, then overwrite
/// it with this frame's) rather than rebuilt from scratch, so steady-state frames don't
/// pay for reallocating and rehashing a ~10k-entry map every single frame. the tradeoff:
/// despawned entities leave a stale entry behind instead of being pruned automatically.
/// fine for now since this engine's scenes are populated once and don't churn entities
pub(crate) fn accumulate_meshes(
    scene: &mut World,
    prev_transforms: &mut HashMap<Entity, Mat4>,
) -> HashMap<MeshSourceId, RenderMeshContainer> {
    let mut containers: HashMap<MeshSourceId, RenderMeshContainer> = HashMap::new();

    for (entity, transform, mesh, material) in scene
        .query::<(Entity, &Transform, &Mesh, Option<&Material>)>()
        .iter(scene)
    {
        let color = material.map(|m| m.color).unwrap_or([1.0, 1.0, 1.0, 1.0]);
        let source_id = *mesh.get_id();
        let model = transform.to_mat4();
        let prev_model = prev_transforms.get(&entity).copied().unwrap_or(model);
        prev_transforms.insert(entity, model);

        if let Some(container) = containers.get_mut(&source_id) {
            container.models.push(model);
            container.prev_models.push(prev_model);
            container.colors.push(color);
        } else {
            containers.insert(
                source_id,
                RenderMeshContainer {
                    id: source_id,
                    models: vec![model],
                    prev_models: vec![prev_model],
                    colors: vec![color],
                },
            );
        }
    }

    containers
}

impl Debug for RenderMeshContainer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(id: {}, count: {})", self.id.0, self.models.len())
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

    #[test]
    fn empty_world_gives_empty_map() {
        let mut world = World::new();
        let containers = accumulate_meshes(&mut world, &mut HashMap::new());
        assert!(containers.is_empty());
    }

    #[test]
    fn single_entity_single_container_single_transform() {
        let mut world = World::new();
        world.spawn((Transform::new(), Mesh::cube()));
        let containers = accumulate_meshes(&mut world, &mut HashMap::new());
        assert_eq!(containers.len(), 1);
        assert_eq!(containers[&MeshSourceId::CUBE].models.len(), 1);
    }

    #[test]
    fn two_same_mesh_one_container_two_transforms() {
        let mut world = World::new();
        world.spawn((Transform::new(), Mesh::cube()));
        world.spawn((Transform::new().with_pos(Vec3::X), Mesh::cube()));
        let containers = accumulate_meshes(&mut world, &mut HashMap::new());
        assert_eq!(containers.len(), 1);
        assert_eq!(containers[&MeshSourceId::CUBE].models.len(), 2);
    }

    #[test]
    fn transform_without_mesh_not_included() {
        let mut world = World::new();
        world.spawn(Transform::new());
        let containers = accumulate_meshes(&mut world, &mut HashMap::new());
        assert!(containers.is_empty());
    }

    #[test]
    fn material_color_stored_per_instance() {
        let mut world = World::new();
        world.spawn((Transform::new(), Mesh::cube(), Material::red()));
        world.spawn((Transform::new(), Mesh::cube(), Material::blue()));
        let containers = accumulate_meshes(&mut world, &mut HashMap::new());
        let container = &containers[&MeshSourceId::CUBE];
        assert_eq!(container.colors.len(), 2);
        // both colors present (order not guaranteed)
        let has_red = container.colors.iter().any(|&c| c == [1.0, 0.0, 0.0, 1.0]);
        let has_blue = container.colors.iter().any(|&c| c == [0.0, 0.0, 1.0, 1.0]);
        assert!(has_red && has_blue);
    }

    #[test]
    fn no_material_defaults_to_white() {
        let mut world = World::new();
        world.spawn((Transform::new(), Mesh::cube()));
        let containers = accumulate_meshes(&mut world, &mut HashMap::new());
        assert_eq!(
            containers[&MeshSourceId::CUBE].colors[0],
            [1.0, 1.0, 1.0, 1.0]
        );
    }

    #[test]
    fn new_entity_has_zero_velocity_prev_equals_current() {
        let mut world = World::new();
        world.spawn((
            Transform::new().with_pos(Vec3::new(5.0, 0.0, 0.0)),
            Mesh::cube(),
        ));
        let containers = accumulate_meshes(&mut world, &mut HashMap::new());
        let container = &containers[&MeshSourceId::CUBE];
        assert_eq!(container.models[0], container.prev_models[0]);
    }

    #[test]
    fn prev_transforms_updated_in_place_feed_next_frame() {
        let mut world = World::new();
        let entity = world.spawn((Transform::new(), Mesh::cube())).id();
        let mut history = HashMap::new();
        accumulate_meshes(&mut world, &mut history);

        {
            let mut transform = world.get_mut::<Transform>(entity).unwrap();
            transform.position = Vec3::new(10.0, 0.0, 0.0);
        }

        let containers = accumulate_meshes(&mut world, &mut history);
        let container = &containers[&MeshSourceId::CUBE];
        assert_ne!(container.models[0], container.prev_models[0]);
    }
}
