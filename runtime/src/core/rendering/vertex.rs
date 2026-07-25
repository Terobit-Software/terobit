use glam::{Vec2, Vec3};

use crate::rendering::{VertexRaw, VertexTrait};

pub(crate) struct Vertex {
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
                    offset: 24, // 16 + 2 floats * 4 bytes = 24
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3, // [f32; 3]
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rendering::VertexTrait;
    use glam::{Vec2, Vec3};

    #[test]
    fn to_raw_position_includes_w_one() {
        let v = Vertex::new(Vec3::new(1.0, 2.0, 3.0), Vec2::ZERO, Vec3::Y);
        assert_eq!(v.to_raw().position, [1.0, 2.0, 3.0, 1.0]);
    }

    #[test]
    fn to_raw_tex_coords_preserved() {
        let v = Vertex::new(Vec3::ZERO, Vec2::new(0.5, 0.75), Vec3::Y);
        assert_eq!(v.to_raw().tex_coords, [0.5, 0.75]);
    }

    #[test]
    fn to_raw_normals_preserved() {
        let v = Vertex::new(Vec3::ZERO, Vec2::ZERO, Vec3::new(0.0, 1.0, 0.0));
        assert_eq!(v.to_raw().normals, [0.0, 1.0, 0.0]);
    }
}
