use std::{collections::HashMap, fmt::Debug};

use bevy_ecs::world::World;

use crate::{ecs::{Mesh, Transform}, rendering::{MeshSourceId, Vertex}};

pub(crate) struct RenderMeshContainer {
    pub id: MeshSourceId,
    pub transforms: Vec<Transform>
}

pub(crate) fn accumulate_meshes (scene: &mut World) -> HashMap<MeshSourceId, RenderMeshContainer> {
    let mut containers: HashMap<MeshSourceId, RenderMeshContainer> = HashMap::new();

    for (transform, mesh) in scene.query::<(&Transform, &Mesh)>().iter(scene) {
        let source_id = mesh.get_id();
        if containers.contains_key(&source_id) {
            if let Some(container) = containers.get_mut(&source_id) {
                container.transforms.push(*transform)
            }
        } else {
            let container = RenderMeshContainer {
                id: *source_id,
                transforms: vec![*transform]
            };

            containers.insert(*source_id, container);
        }
    };

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
        world.spawn(Transform::new()); // no Mesh component
        assert!(accumulate_meshes(&mut world).is_empty());
    }
}