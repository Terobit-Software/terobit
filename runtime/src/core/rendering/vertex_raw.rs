#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VertexRaw {
    pub position: [f32; 4],   // x, y, z, and 1.0 for some reason
    pub tex_coords: [f32; 2], // u, v
    pub normals: [f32; 3],
}

impl VertexRaw {
    pub fn new(position: [f32; 4], tex_coords: [f32; 2], normals: [f32; 3]) -> Self {
        VertexRaw {
            position,
            tex_coords,
            normals,
        }
    }
}
