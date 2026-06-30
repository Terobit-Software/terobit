use glam::Mat4;

use crate::ecs::Transform;

/// per-instance model matrix uploaded to the gpu, one per entity with a mesh
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    model: [[f32; 4]; 4],
}

impl InstanceRaw {
    /// packs a transform into the model matrix format the shader expects (locations 3–6)
    pub fn new(transform: Transform) -> Self {
        Self { model: transform.to_mat4().to_cols_array_2d() }
    }

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance, // advances once per instance
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 3,           // locations 3-6, after vertex attrs 0-2
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: 16,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: 32,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: 48,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::{Mat4, Vec3};
    use crate::ecs::Transform;

    #[test]
    fn identity_transform_produces_identity_matrix() {
        let raw = InstanceRaw::new(Transform::new());
        assert_eq!(raw.model, Mat4::IDENTITY.to_cols_array_2d());
    }

    #[test]
    fn translation_lands_in_column_3() {
        let raw = InstanceRaw::new(Transform::new().with_pos(Vec3::new(1.0, 2.0, 3.0)));
        // column-major layout: col 3 = raw.model[3]
        assert!((raw.model[3][0] - 1.0).abs() < f32::EPSILON);
        assert!((raw.model[3][1] - 2.0).abs() < f32::EPSILON);
        assert!((raw.model[3][2] - 3.0).abs() < f32::EPSILON);
        assert!((raw.model[3][3] - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn uniform_scale_on_diagonal() {
        let raw = InstanceRaw::new(Transform::new().with_scale(Vec3::splat(3.0)));
        assert!((raw.model[0][0] - 3.0).abs() < f32::EPSILON);
        assert!((raw.model[1][1] - 3.0).abs() < f32::EPSILON);
        assert!((raw.model[2][2] - 3.0).abs() < f32::EPSILON);
    }
}