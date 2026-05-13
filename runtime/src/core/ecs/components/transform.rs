use glam::{Mat4, Quat, Vec3};

#[derive(bevy_ecs::component::Component)]
pub struct Transform {
    position: Vec3,
    rotation: Quat,
    scale: Vec3,
}

impl Transform {
    pub fn to_mat4(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }

    pub fn new () -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }

    pub fn with_pos (mut self, position: Vec3) -> Self {
        self.position = position;
        self
    }

    pub fn with_rot (mut self, rotation: Quat) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn with_scale (mut self, scale: Vec3) -> Self {
        self.scale = scale;
        self
    }

}

impl Default for Transform {
    fn default() -> Self {
        Self::new()
    }
}
