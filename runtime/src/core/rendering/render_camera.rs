use glam::{Mat4, Vec3};

pub(crate) struct RenderCamera {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
}

impl RenderCamera {
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

    pub fn build_view_proj(&self, width: u32, height: u32) -> [[f32; 4]; 4] {
        let view = Mat4::look_at_rh(self.position, self.target, self.up);
        let projection =
            Mat4::perspective_rh(self.fov, width as f32 / height as f32, self.near, self.far);
        (projection * view).to_cols_array_2d()
    }
}
