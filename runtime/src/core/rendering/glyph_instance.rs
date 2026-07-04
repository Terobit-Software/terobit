/// one glyph quad on screen: pixel position/size plus its atlas uv rect and color.
/// drawn instanced against a shared unit quad, screen_pos is the top-left corner
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct GlyphInstance {
    pub screen_pos: [f32; 2],
    pub size: [f32; 2],
    pub uv_min: [f32; 2],
    pub uv_max: [f32; 2],
    pub color: [f32; 4],
}

impl GlyphInstance {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 8,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 16,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 24,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 32,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

/// unit quad corner, the non-instanced base geometry every glyph is stamped from
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct QuadVertex {
    pub corner: [f32; 2],
}

impl QuadVertex {
    pub const VERTICES: [QuadVertex; 4] = [
        QuadVertex { corner: [0.0, 0.0] },
        QuadVertex { corner: [1.0, 0.0] },
        QuadVertex { corner: [0.0, 1.0] },
        QuadVertex { corner: [1.0, 1.0] },
    ];
    pub const INDICES: [u16; 6] = [0, 1, 2, 2, 1, 3];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x2,
            }],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glyph_instance_size_is_48_bytes() {
        // screen_pos[2] + size[2] + uv_min[2] + uv_max[2] + color[4] = 12 floats
        assert_eq!(std::mem::size_of::<GlyphInstance>(), 48);
    }

    #[test]
    fn quad_has_four_corners_two_triangles() {
        assert_eq!(QuadVertex::VERTICES.len(), 4);
        assert_eq!(QuadVertex::INDICES.len(), 6);
    }
}
