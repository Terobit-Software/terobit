use std::{collections::HashMap, fmt::Debug};

use bevy_ecs::world::World;

use crate::{ecs::{Mesh, Transform}, rendering::{MeshSourceId, RenderMesh, Vertex}};

pub(crate) struct RenderMeshContainer {
    id: MeshSourceId,
    transforms: Vec<Transform>
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