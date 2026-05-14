use std::collections::HashMap;

use bevy_ecs::world::World;

use crate::{ecs::{Mesh, Transform}, rendering::{MeshSourceId, RenderMesh, Vertex}};

pub(crate) struct RenderMeshContainer {
    mesh: MeshSourceId,
    transforms: Vec<Transform>
}

pub(crate) fn accumulate_meshes (scene: &mut World) -> HashMap<MeshSourceId, RenderMeshContainer> {
    let mut containers: HashMap<u32, RenderMeshContainer> = HashMap::new();

    for (transform, mesh) in scene.query::<(&Transform, &Mesh)>().iter(scene) {
        let source_id = mesh.get_source().get_id();
        if containers.contains_key(&source_id) {
            if let Some(container) = containers.get_mut(&source_id) {
                container.transforms.push(*transform)
            }
        } else {
            let container = RenderMeshContainer {
                mesh: source_id,
                transforms: vec![*transform]
            };

            containers.insert(source_id, container);
        }
    };

    containers
}