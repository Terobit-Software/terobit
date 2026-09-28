use bytemuck::{Pod, Zeroable};

use crate::assets::asset_error::AssetError;
use crate::assets::raw_asset::RawAssetData;
use crate::assets::registry::{AssetId, AssetRegistry};
use crate::consts::mesh::CUBE_FACES;
use tracing::info;

/// A single mesh vertex: position and face normal, both in model space.
/// `Pod` so a mesh's vertices can be uploaded to the GPU as-is.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(crate) struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}

/// Parsed mesh geometry. Owned exclusively by the [`AssetRegistry`]; nothing
/// outside the registry holds this data directly.
pub(crate) struct MeshData {
    verts: Vec<Vertex>,
    indices: Vec<u16>,
    radius: f32,
}

impl MeshData {
    pub(crate) fn from_raw(raw: &RawAssetData) -> Result<Self, AssetError> {
        if raw.is_builtin() {
            return Ok(Self::builtin_cube());
        }

        Self::parse_obj(raw.data())
    }

    #[inline]
    pub(crate) fn verts(&self) -> &[Vertex] {
        &self.verts
    }

    #[inline]
    pub(crate) fn indices(&self) -> &[u16] {
        &self.indices
    }

    /// Unit cube (extents -0.5..0.5 on each axis), 4 duplicated vertices per
    /// face so each face gets its own flat normal instead of smoothed corner
    /// normals.
    fn builtin_cube() -> Self {
        let mut verts = Vec::with_capacity(24);
        let mut indices = Vec::with_capacity(36);

        for (normal, corners) in CUBE_FACES {
            let base = verts.len() as u16;
            for corner in corners {
                verts.push(Vertex {
                    position: corner,
                    normal,
                });
            }
            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }

        let mut mesh = Self {
            verts,
            indices,
            radius: 0.0,
        };

        mesh.update_radius();

        info!(
            "Created default cube mesh with {} vertices, {} indices, radius {}",
            mesh.verts.len(),
            mesh.indices.len(),
            mesh.radius
        );

        mesh
    }

    #[inline]
    fn radius(&self) -> f32 {
        self.radius
    }

    fn update_radius(&mut self) {
        let mut max = 0.0;
        for vertex in &self.verts {
            let dist_squared = vertex.position[0] * vertex.position[0]
                + vertex.position[1] * vertex.position[1]
                + vertex.position[2] * vertex.position[2];
            if dist_squared > max {
                max = dist_squared;
            }
        }
        self.radius = max.sqrt();
    }

    /// Parses an OBJ file via `tobj`, triangulating faces and merging
    /// duplicate vertices into a single index buffer. Meshes that don't
    /// specify `vn` normals get smooth per-vertex normals computed from the
    /// surrounding triangles instead.
    fn parse_obj(data: &[u8]) -> Result<Self, AssetError> {
        let mut reader = std::io::BufReader::new(data);

        let (models, _) = tobj::load_obj_buf(&mut reader, &tobj::GPU_LOAD_OPTIONS, |_| {
            Err(tobj::LoadError::OpenFileFailed)
        })
        .map_err(|e| AssetError::InvalidMeshData(e.to_string()))?;

        let Some(model) = models.into_iter().next() else {
            return Err(AssetError::InvalidMeshData(
                "mesh contains no faces".to_string(),
            ));
        };
        let mesh = model.mesh;

        let vertex_count = mesh.positions.len() / 3;
        if vertex_count > u16::MAX as usize + 1 {
            return Err(AssetError::InvalidMeshData(
                "mesh exceeds the 65536 vertex limit".to_string(),
            ));
        }

        let has_normals = mesh.normals.len() == mesh.positions.len();
        let mut verts: Vec<Vertex> = (0..vertex_count)
            .map(|i| Vertex {
                position: [
                    mesh.positions[i * 3],
                    mesh.positions[i * 3 + 1],
                    mesh.positions[i * 3 + 2],
                ],
                normal: if has_normals {
                    [
                        mesh.normals[i * 3],
                        mesh.normals[i * 3 + 1],
                        mesh.normals[i * 3 + 2],
                    ]
                } else {
                    [0.0, 0.0, 0.0]
                },
            })
            .collect();

        let indices: Vec<u16> = mesh.indices.iter().map(|&i| i as u16).collect();

        if verts.is_empty() {
            return Err(AssetError::InvalidMeshData(
                "mesh contains no faces".to_string(),
            ));
        }

        if !has_normals {
            compute_smooth_normals(&mut verts, &indices);
        }

        let mut obj = Self {
            verts,
            indices,
            radius: 0.0,
        };

        obj.update_radius();

        info!(
            "Parsed mesh with {} vertices, {} indices, radius {}",
            obj.verts.len(),
            obj.indices.len(),
            obj.radius
        );

        Ok(obj)
    }
}

/// A handle to mesh geometry stored in the [`AssetRegistry`]. Holds only an
/// [`AssetId`] — every getter takes the registry and looks the data up
/// there.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct MeshAsset {
    id: AssetId,
}

impl MeshAsset {
    #[inline]
    pub(crate) fn new(id: AssetId) -> Self {
        Self { id }
    }

    #[inline]
    pub(crate) fn id(&self) -> AssetId {
        self.id
    }

    #[inline]
    pub(crate) fn verts<'a>(&self, registry: &'a AssetRegistry) -> &'a [Vertex] {
        registry.mesh_verts(self.id)
    }

    #[inline]
    pub(crate) fn indices<'a>(&self, registry: &'a AssetRegistry) -> &'a [u16] {
        registry.mesh_indices(self.id)
    }
}

/// Fills in per-vertex normals for meshes that didn't specify their own,
/// by averaging the face normal of every triangle each vertex touches.
fn compute_smooth_normals(verts: &mut [Vertex], indices: &[u16]) {
    let mut accum = vec![[0.0f32; 3]; verts.len()];

    for &[a, b, c] in indices.as_chunks::<3>().0 {
        let [a, b, c] = [a as usize, b as usize, c as usize];
        let normal = cross(
            sub(verts[b].position, verts[a].position),
            sub(verts[c].position, verts[a].position),
        );
        for i in [a, b, c] {
            accum[i] = add(accum[i], normal);
        }
    }

    for (vertex, normal) in verts.iter_mut().zip(accum) {
        vertex.normal = normalize(normal);
    }
}

fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len <= f32::EPSILON {
        return [0.0, 0.0, 0.0];
    }
    [v[0] / len, v[1] / len, v[2] / len]
}
