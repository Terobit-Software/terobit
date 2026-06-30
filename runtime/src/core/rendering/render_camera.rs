use glam::{Mat4, Vec3};

/// camera data used to build the view-projection matrix each frame
pub(crate) struct RenderCamera {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
}

impl RenderCamera {
    /// creates a new camera looking at `target` from `position`. fov=45°, near=0.1, far=10000
    pub fn new(position: Vec3, target: Vec3) -> Self {
        Self {
            position,
            target,
            up: Vec3::Y,
            fov: 45f32.to_radians(),
            near: 0.1,
            far: 10000.0,
        }
    }

    /// builds the combined view-projection matrix for the given surface size.
    /// returns column-major f32 array ready for the camera uniform buffer
    pub fn build_view_proj(&self, width: u32, height: u32) -> [[f32; 4]; 4] {
        let view = Mat4::look_at_rh(self.position, self.target, self.up);
        let projection =
            Mat4::perspective_rh(self.fov, width as f32 / height as f32, self.near, self.far);
        (projection * view).to_cols_array_2d()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;

    #[test]
    fn new_has_correct_defaults() {
        let cam = RenderCamera::new(Vec3::ZERO, Vec3::X);
        assert_eq!(cam.up, Vec3::Y);
        assert!((cam.fov - 45f32.to_radians()).abs() < f32::EPSILON);
        assert!((cam.near - 0.1).abs() < f32::EPSILON);
        assert!((cam.far - 10000.0).abs() < f32::EPSILON);
    }

    #[test]
    fn build_view_proj_all_values_finite() {
        let cam = RenderCamera::new(Vec3::new(0.0, 0.0, -5.0), Vec3::ZERO);
        let mat = cam.build_view_proj(1280, 720);
        for col in &mat {
            for val in col {
                assert!(val.is_finite(), "non-finite value in view_proj: {val}");
            }
        }
    }

    #[test]
    fn build_view_proj_differs_by_aspect_ratio() {
        let cam = RenderCamera::new(Vec3::new(0.0, 0.0, -5.0), Vec3::ZERO);
        let wide = cam.build_view_proj(1920, 1080);
        let square = cam.build_view_proj(512, 512);
        assert_ne!(wide, square);
    }
}
