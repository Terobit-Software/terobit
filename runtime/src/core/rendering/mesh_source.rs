use std::collections::HashMap;

use glam::{Vec2, Vec3};

use crate::{app::AppState, rendering::{Vertex, VertexRaw, VertexTrait}};

/// unique id for a mesh type. used to look up gpu buffers in the mesh registry
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MeshSourceId(pub usize);

impl MeshSourceId {
    /// built-in cube mesh id. custom meshes start at 2 and increment
    pub const CUBE: Self = Self(0);
    /// built-in flat plane mesh id (unit quad, xz plane, facing +y)
    pub const PLANE: Self = Self(1);
}

impl From<MeshSourceId> for usize {
    fn from(id: MeshSourceId) -> usize { id.0 }
}

impl From<usize> for MeshSourceId {
    fn from(n: usize) -> Self { Self(n) }
}

pub(crate) struct MeshSource {
    id: MeshSourceId,
}

pub(crate) struct MeshSourceHandler {
    meshes: HashMap<MeshSourceId, MeshSource>,
}

impl MeshSourceHandler {
    pub fn init_default_meshes(state: &mut AppState) -> Result<(), &'static str> {
        let (cube_vertices, cube_indices) = Self::cube_mesh();
        state.upload_mesh(MeshSourceId::CUBE, &Self::format_verts(cube_vertices), &cube_indices);

        let (plane_vertices, plane_indices) = Self::plane_mesh();
        state.upload_mesh(MeshSourceId::PLANE, &Self::format_verts(plane_vertices), &plane_indices);

        Ok(())
    }

    /// unit flat quad in the xz plane, centered at origin, facing +y
    fn plane_mesh() -> (Vec<Vertex>, Vec<u32>) {
        let h = 0.5_f32;
        let normal = Vec3::new(0.0, 1.0, 0.0);

        let vertices = vec![
            Vertex::new(Vec3::new(-h, 0.0, -h), Vec2::new(0.0, 0.0), normal),
            Vertex::new(Vec3::new(h, 0.0, -h), Vec2::new(1.0, 0.0), normal),
            Vertex::new(Vec3::new(-h, 0.0, h), Vec2::new(0.0, 1.0), normal),
            Vertex::new(Vec3::new(h, 0.0, h), Vec2::new(1.0, 1.0), normal),
        ];

        let indices = vec![0, 1, 2, 2, 1, 3];

        (vertices, indices)
    }

    fn format_verts(vertices: Vec<Vertex>) -> Vec<VertexRaw> {
        vertices.iter().map(|v| v.to_raw()).collect()
    }

    /// Unit cube centered at origin.
    /// Each face has its own 4 vertices so normals are hard (flat), not shared.
    fn cube_mesh() -> (Vec<Vertex>, Vec<u32>) {
        let h = 0.5_f32; // half-extent

        // Each face: 4 verts (tl, tr, bl, br), normal pointing outward
        // UV: simple 0..1 across each face
        let faces: &[(Vec3, Vec3, Vec3, Vec3, Vec3)] = &[
            // (top-left pos, top-right pos, bot-left pos, bot-right pos, normal)

            // +Z front
            (
                Vec3::new(-h,  h,  h),
                Vec3::new( h,  h,  h),
                Vec3::new(-h, -h,  h),
                Vec3::new( h, -h,  h),
                Vec3::new(0.0, 0.0, 1.0),
            ),
            // -Z back
            (
                Vec3::new( h,  h, -h),
                Vec3::new(-h,  h, -h),
                Vec3::new( h, -h, -h),
                Vec3::new(-h, -h, -h),
                Vec3::new(0.0, 0.0, -1.0),
            ),
            // +X right
            (
                Vec3::new( h,  h,  h),
                Vec3::new( h,  h, -h),
                Vec3::new( h, -h,  h),
                Vec3::new( h, -h, -h),
                Vec3::new(1.0, 0.0, 0.0),
            ),
            // -X left
            (
                Vec3::new(-h,  h, -h),
                Vec3::new(-h,  h,  h),
                Vec3::new(-h, -h, -h),
                Vec3::new(-h, -h,  h),
                Vec3::new(-1.0, 0.0, 0.0),
            ),
            // +Y top
            (
                Vec3::new(-h,  h, -h),
                Vec3::new( h,  h, -h),
                Vec3::new(-h,  h,  h),
                Vec3::new( h,  h,  h),
                Vec3::new(0.0, 1.0, 0.0),
            ),
            // -Y bottom
            (
                Vec3::new(-h, -h,  h),
                Vec3::new( h, -h,  h),
                Vec3::new(-h, -h, -h),
                Vec3::new( h, -h, -h),
                Vec3::new(0.0, -1.0, 0.0),
            ),
        ];

        let mut vertices: Vec<Vertex> = Vec::with_capacity(24); // 6 faces * 4 verts
        let mut indices: Vec<u32>     = Vec::with_capacity(36); // 6 faces * 2 tris * 3 verts

        for (i, (tl, tr, bl, br, normal)) in faces.iter().enumerate() {
            let base = (i * 4) as u32;

            vertices.push(Vertex::new(*tl, Vec2::new(0.0, 0.0), *normal));
            vertices.push(Vertex::new(*tr, Vec2::new(1.0, 0.0), *normal));
            vertices.push(Vertex::new(*bl, Vec2::new(0.0, 1.0), *normal));
            vertices.push(Vertex::new(*br, Vec2::new(1.0, 1.0), *normal));

            // Two CCW triangles per face:
            // tl--tr
            // |  / |
            // bl--br
            indices.extend_from_slice(&[
                base,     base + 1, base + 2, // tl, tr, bl
                base + 2, base + 1, base + 3, // bl, tr, br
            ]);
        }

        (vertices, indices)
    }
}

impl MeshSource {
    pub fn cube() -> Self {
        Self { id: MeshSourceId::CUBE }
    }

    pub(crate) fn get_id(&self) -> MeshSourceId {
        self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rendering::VertexTrait;

    #[test]
    fn cube_constant_is_zero() {
        assert_eq!(MeshSourceId::CUBE, MeshSourceId(0));
    }

    #[test]
    fn plane_constant_is_one() {
        assert_eq!(MeshSourceId::PLANE, MeshSourceId(1));
    }

    #[test]
    fn plane_mesh_has_4_vertices_and_6_indices() {
        let (verts, indices) = MeshSourceHandler::plane_mesh();
        assert_eq!(verts.len(), 4);
        assert_eq!(indices.len(), 6);
    }

    #[test]
    fn from_usize_roundtrip() {
        let id = MeshSourceId::from(42_usize);
        let back: usize = id.into();
        assert_eq!(back, 42);
    }

    #[test]
    fn cube_mesh_has_24_vertices() {
        // 6 faces * 4 verts, hard normals so no sharing
        let (verts, _) = MeshSourceHandler::cube_mesh();
        assert_eq!(verts.len(), 24);
    }

    #[test]
    fn cube_mesh_has_36_indices() {
        // 6 faces * 2 tris * 3 verts
        let (_, indices) = MeshSourceHandler::cube_mesh();
        assert_eq!(indices.len(), 36);
    }

    #[test]
    fn cube_mesh_indices_in_range() {
        let (verts, indices) = MeshSourceHandler::cube_mesh();
        let max = verts.len() as u32;
        for &i in &indices {
            assert!(i < max, "index {i} out of bounds (vertex count {max})");
        }
    }

    #[test]
    fn cube_mesh_normals_are_unit_length() {
        let (verts, _) = MeshSourceHandler::cube_mesh();
        for v in &verts {
            let [nx, ny, nz] = v.to_raw().normals;
            let len = (nx * nx + ny * ny + nz * nz).sqrt();
            assert!((len - 1.0).abs() < 1e-5, "normal not unit: ({nx}, {ny}, {nz})");
        }
    }
}