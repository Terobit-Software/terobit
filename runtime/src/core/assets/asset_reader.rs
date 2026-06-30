use std::path::Path;

use anyhow::Context;

use crate::rendering::VertexRaw;

/// loads and parses asset files from disk
pub struct AssetReader;

impl AssetReader {
    /// creates a new asset reader
    pub fn new() -> Self {
        AssetReader
    }

    /// loads an obj file and returns (vertices, indices) ready for gpu upload.
    /// missing normals default to (0, 1, 0), missing texcoords default to (0, 0).
    /// all meshes in the file are merged into one vertex/index buffer.
    pub fn load_obj(path: &Path) -> anyhow::Result<(Vec<VertexRaw>, Vec<u32>)> {
        let (models, _) = tobj::load_obj(
            path,
            &tobj::LoadOptions {
                triangulate: true,
                // single_index makes all attributes share one index buffer,
                // which is what our VertexRaw layout expects
                single_index: true,
                ..Default::default()
            },
        )
        .with_context(|| format!("failed to load obj: {:?}", path))?;

        let mut all_vertices: Vec<VertexRaw> = Vec::new();
        let mut all_indices: Vec<u32> = Vec::new();
        let mut index_offset: u32 = 0;

        for model in &models {
            let mesh = &model.mesh;
            let vertex_count = mesh.positions.len() / 3;

            for i in 0..vertex_count {
                let position = [
                    mesh.positions[i * 3],
                    mesh.positions[i * 3 + 1],
                    mesh.positions[i * 3 + 2],
                    1.0, // w=1.0 as the shader expects vec4
                ];

                let tex_coords = if mesh.texcoords.len() >= (i + 1) * 2 {
                    [mesh.texcoords[i * 2], mesh.texcoords[i * 2 + 1]]
                } else {
                    [0.0, 0.0]
                };

                let normals = if mesh.normals.len() >= (i + 1) * 3 {
                    [
                        mesh.normals[i * 3],
                        mesh.normals[i * 3 + 1],
                        mesh.normals[i * 3 + 2],
                    ]
                } else {
                    [0.0, 1.0, 0.0]
                };

                all_vertices.push(VertexRaw { position, tex_coords, normals });
            }

            for &idx in &mesh.indices {
                all_indices.push(idx + index_offset);
            }

            index_offset += vertex_count as u32;
        }

        Ok((all_vertices, all_indices))
    }
}

impl Default for AssetReader {
    fn default() -> Self {
        Self::new()
    }
}
