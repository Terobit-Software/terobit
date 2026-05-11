use glam::{Vec2, Vec3};

use crate::rendering::{VertexRaw, VertexTrait};

pub struct Vertex {
    position: Vec3,
    tex_coords: Vec2,
    normal: Vec3,
}

impl Vertex {
    pub fn new(position: Vec3, tex_coords: Vec2, normal: Vec3) -> Self {
        Self {
            position,
            tex_coords,
            normal,
        }
    }
}

impl VertexTrait for Vertex {
    type Raw = VertexRaw;

    fn to_raw(&self) -> Self::Raw {
        Self::Raw {
            position: [self.position.x, self.position.y, self.position.z, 1.0],
            tex_coords: [self.tex_coords.x, self.tex_coords.y],
            normals: [self.normal.x, self.normal.y, self.normal.z],
        }
    }

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self::Raw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x4, // [f32; 4]
                },
                wgpu::VertexAttribute {
                    offset: 16, // 4 floats * 4 bytes = 16
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2, // [f32; 2]
                },
                wgpu::VertexAttribute {
                    offset: 24,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}
