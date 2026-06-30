use glam::Mat4;

use crate::ecs::Transform;

/// per-instance model matrix + color uploaded to the gpu, one per entity with a mesh
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    model: [[f32; 4]; 4],
    /// rgba, passed straight to the fragment shader as the base color
    color: [f32; 4],
}

impl InstanceRaw {
    /// packs a transform and color into the instance format the shader expects (locations 3-7)
    pub fn new(transform: Transform, color: [f32; 4]) -> Self {
        Self {
            model: transform.to_mat4().to_cols_array_2d(),
            color,
        }
    }

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 3,
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
                // color rgba at location 7, right after the 64-byte model matrix
                wgpu::VertexAttribute {
                    offset: 64,
                    shader_location: 7,
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

    const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

    #[test]
    fn identity_transform_produces_identity_matrix() {
        let raw = InstanceRaw::new(Transform::new(), WHITE);
        assert_eq!(raw.model, Mat4::IDENTITY.to_cols_array_2d());
    }

    #[test]
    fn translation_lands_in_column_3() {
        let raw = InstanceRaw::new(Transform::new().with_pos(Vec3::new(1.0, 2.0, 3.0)), WHITE);
        assert!((raw.model[3][0] - 1.0).abs() < f32::EPSILON);
        assert!((raw.model[3][1] - 2.0).abs() < f32::EPSILON);
        assert!((raw.model[3][2] - 3.0).abs() < f32::EPSILON);
        assert!((raw.model[3][3] - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn uniform_scale_on_diagonal() {
        let raw = InstanceRaw::new(Transform::new().with_scale(Vec3::splat(3.0)), WHITE);
        assert!((raw.model[0][0] - 3.0).abs() < f32::EPSILON);
        assert!((raw.model[1][1] - 3.0).abs() < f32::EPSILON);
        assert!((raw.model[2][2] - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn color_stored_correctly() {
        let color = [1.0, 0.5, 0.0, 1.0];
        let raw = InstanceRaw::new(Transform::new(), color);
        assert_eq!(raw.color, color);
    }
}
