/// packed vertex data in the exact layout the shader expects
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct VertexRaw {
    pub position: [f32; 4],   // x, y, z, w=1.0 (shader reads as vec4)
    pub tex_coords: [f32; 2], // u, v
    pub normals: [f32; 3],
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_is_36_bytes() {
        // position [f32;4]=16, tex_coords [f32;2]=8, normals [f32;3]=12 → 36
        assert_eq!(std::mem::size_of::<VertexRaw>(), 36);
    }
}
