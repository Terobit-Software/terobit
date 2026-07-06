use glam::Mat4;

/// per-instance model matrix + color uploaded to the gpu, one per entity with a mesh.
/// carries last frame's model matrix too so the shader can compute per-pixel motion
/// vectors for taa's temporal reprojection
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    model: [[f32; 4]; 4],
    prev_model: [[f32; 4]; 4],
    /// rgba, passed straight to the fragment shader as the base color
    color: [f32; 4],
}

impl InstanceRaw {
    /// packs this frame's and last frame's model matrices plus a color into the
    /// instance format the shader expects (locations 3-11)
    pub fn new(model: Mat4, prev_model: Mat4, color: [f32; 4]) -> Self {
        Self {
            model: model.to_cols_array_2d(),
            prev_model: prev_model.to_cols_array_2d(),
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
                // prev_model right after the 64-byte model matrix, locations 7-10
                wgpu::VertexAttribute {
                    offset: 64,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: 80,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: 96,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: 112,
                    shader_location: 10,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // color rgba at location 11, right after both 64-byte matrices
                wgpu::VertexAttribute {
                    offset: 128,
                    shader_location: 11,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;

    const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

    #[test]
    fn identity_transform_produces_identity_matrix() {
        let raw = InstanceRaw::new(Mat4::IDENTITY, Mat4::IDENTITY, WHITE);
        assert_eq!(raw.model, Mat4::IDENTITY.to_cols_array_2d());
    }

    #[test]
    fn translation_lands_in_column_3() {
        let model = Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0));
        let raw = InstanceRaw::new(model, Mat4::IDENTITY, WHITE);
        assert!((raw.model[3][0] - 1.0).abs() < f32::EPSILON);
        assert!((raw.model[3][1] - 2.0).abs() < f32::EPSILON);
        assert!((raw.model[3][2] - 3.0).abs() < f32::EPSILON);
        assert!((raw.model[3][3] - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn uniform_scale_on_diagonal() {
        let model = Mat4::from_scale(Vec3::splat(3.0));
        let raw = InstanceRaw::new(model, Mat4::IDENTITY, WHITE);
        assert!((raw.model[0][0] - 3.0).abs() < f32::EPSILON);
        assert!((raw.model[1][1] - 3.0).abs() < f32::EPSILON);
        assert!((raw.model[2][2] - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn color_stored_correctly() {
        let color = [1.0, 0.5, 0.0, 1.0];
        let raw = InstanceRaw::new(Mat4::IDENTITY, Mat4::IDENTITY, color);
        assert_eq!(raw.color, color);
    }

    #[test]
    fn prev_model_stored_separately_from_model() {
        let model = Mat4::from_translation(Vec3::new(1.0, 0.0, 0.0));
        let prev_model = Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0));
        let raw = InstanceRaw::new(model, prev_model, WHITE);
        assert_ne!(raw.model, raw.prev_model);
        assert_eq!(raw.prev_model, prev_model.to_cols_array_2d());
    }
}
