use std::collections::HashMap;

use glam::{Vec2, Vec3};

use crate::{app::AppState, assets::AssetReference, ecs::Mesh, rendering::{Vertex, VertexRaw, VertexTrait}};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MeshSourceId(pub usize);

impl MeshSourceId {
    pub fn from_reference(asset: AssetReference) -> Self {
        Self(0)
    }
    pub const CUBE: Self = Self(0);
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
        let (vertices, indices) = Self::cube_mesh();
    
        state.upload_mesh(MeshSourceId::CUBE, &Self::format_verts(vertices), &indices);
        Ok(())
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

    pub fn from_reference(asset: AssetReference) -> Self {
        Self { id: MeshSourceId(0) }
    }

    pub fn from_id(id: MeshSourceId) -> Self {
        Self { id }
    }

    pub(crate) fn get_id(&self) -> MeshSourceId {
        self.id
    }
}