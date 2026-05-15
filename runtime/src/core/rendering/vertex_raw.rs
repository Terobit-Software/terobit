#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct VertexRaw {
    pub position: [f32; 4],   // x, y, z, and 1.0 for some reason
    pub tex_coords: [f32; 2], // u, v
    pub normals: [f32; 3],
}