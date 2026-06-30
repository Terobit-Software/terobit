use std::{collections::HashMap, fmt::Debug};

use bevy_ecs::world::World;

use crate::{ecs::{Material, Mesh, Transform}, rendering::MeshSourceId};

pub(crate) struct RenderMeshContainer {
    pub id: MeshSourceId,
    /// transforms and colors are parallel, one entry per instance
    pub transforms: Vec<Transform>,
    pub colors: Vec<[f32; 4]>,
}

/// groups all (Transform, Mesh) entities by mesh id, collecting per-instance colors from
/// optional Material components. entities without Material get white as the default
pub(crate) fn accumulate_meshes(scene: &mut World) -> HashMap<MeshSourceId, RenderMeshContainer> {
    let mut containers: HashMap<MeshSourceId, RenderMeshContainer> = HashMap::new();

    for (transform, mesh, material) in scene
        .query::<(&Transform, &Mesh, Option<&Material>)>()
        .iter(scene)
    {
        let color = material.map(|m| m.color).unwrap_or([1.0, 1.0, 1.0, 1.0]);
        let source_id = mesh.get_id();

        if let Some(container) = containers.get_mut(&source_id) {
            container.transforms.push(*transform);
            container.colors.push(color);
        } else {
            containers.insert(*source_id, RenderMeshContainer {
                id: *source_id,
                transforms: vec![*transform],
                colors: vec![color],
            });
        }
    }

    containers
}

impl Debug for RenderMeshContainer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(id: {}, count: {})", self.id.0, self.transforms.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;
    use crate::{ecs::{Mesh, Transform}, rendering::MeshSourceId};

    #[test]
    fn empty_world_gives_empty_map() {
        let mut world = World::new();
        assert!(accumulate_meshes(&mut world).is_empty());
    }

    #[test]
    fn single_entity_single_container_single_transform() {
        let mut world = World::new();
        world.spawn((Transform::new(), Mesh::cube()));
        let result = accumulate_meshes(&mut world);
        assert_eq!(result.len(), 1);
        assert_eq!(result[&MeshSourceId::CUBE].transforms.len(), 1);
    }

    #[test]
    fn two_same_mesh_one_container_two_transforms() {
        let mut world = World::new();
        world.spawn((Transform::new(), Mesh::cube()));
        world.spawn((Transform::new().with_pos(Vec3::X), Mesh::cube()));
        let result = accumulate_meshes(&mut world);
        assert_eq!(result.len(), 1);
        assert_eq!(result[&MeshSourceId::CUBE].transforms.len(), 2);
    }

    #[test]
    fn transform_without_mesh_not_included() {
        let mut world = World::new();
        world.spawn(Transform::new());
        assert!(accumulate_meshes(&mut world).is_empty());
    }

    #[test]
    fn material_color_stored_per_instance() {
        let mut world = World::new();
        world.spawn((Transform::new(), Mesh::cube(), Material::red()));
        world.spawn((Transform::new(), Mesh::cube(), Material::blue()));
        let result = accumulate_meshes(&mut world);
        let container = &result[&MeshSourceId::CUBE];
        assert_eq!(container.colors.len(), 2);
        // both colors present (order not guaranteed)
        let has_red  = container.colors.iter().any(|&c| c == [1.0, 0.0, 0.0, 1.0]);
        let has_blue = container.colors.iter().any(|&c| c == [0.0, 0.0, 1.0, 1.0]);
        assert!(has_red && has_blue);
    }

    #[test]
    fn no_material_defaults_to_white() {
        let mut world = World::new();
        world.spawn((Transform::new(), Mesh::cube()));
        let result = accumulate_meshes(&mut world);
        assert_eq!(result[&MeshSourceId::CUBE].colors[0], [1.0, 1.0, 1.0, 1.0]);
    }
}
