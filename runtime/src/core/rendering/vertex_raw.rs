#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VertexRaw {
    pub position: [f32; 4], // x, y, z, and 1.0 for some reason
    pub tex_coords: [f32; 2], // u, v
}

impl VertexRaw {
    pub fn new (position: [f32; 4], tex_coords: [f32; 2]) -> Self {
        VertexRaw { 
            position, 
            tex_coords 
        }
    }
}