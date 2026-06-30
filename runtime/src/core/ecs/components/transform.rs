use glam::{Mat4, Quat, Vec3};

/// position, rotation, and scale of an entity in world space
#[derive(bevy_ecs::component::Component, Clone, Copy)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Transform {
    /// converts to a 4x4 model matrix (scale → rotate → translate order)
    pub fn to_mat4(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }

    /// creates a transform at the origin with identity rotation and unit scale
    pub fn new () -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }

    /// sets position, returns self (builder style)
    pub fn with_pos (mut self, position: Vec3) -> Self {
        self.position = position;
        self
    }

    /// sets rotation, returns self (builder style)
    pub fn with_rot (mut self, rotation: Quat) -> Self {
        self.rotation = rotation;
        self
    }

    /// sets scale, returns self (builder style)
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

#[cfg(test)]
mod tests {
    use super::*;
    use glam::{Mat4, Quat, Vec3};

    #[test]
    fn new_is_identity_state() {
        let t = Transform::new();
        assert_eq!(t.position, Vec3::ZERO);
        assert_eq!(t.rotation, Quat::IDENTITY);
        assert_eq!(t.scale, Vec3::ONE);
    }

    #[test]
    fn default_matches_new() {
        let t = Transform::default();
        assert_eq!(t.position, Vec3::ZERO);
        assert_eq!(t.rotation, Quat::IDENTITY);
        assert_eq!(t.scale, Vec3::ONE);
    }

    #[test]
    fn with_pos_sets_position() {
        let pos = Vec3::new(1.0, 2.0, 3.0);
        let t = Transform::new().with_pos(pos);
        assert_eq!(t.position, pos);
    }

    #[test]
    fn with_rot_sets_rotation() {
        let rot = Quat::from_rotation_y(std::f32::consts::PI);
        let t = Transform::new().with_rot(rot);
        assert_eq!(t.rotation, rot);
    }

    #[test]
    fn with_scale_sets_scale() {
        let scale = Vec3::new(2.0, 3.0, 4.0);
        let t = Transform::new().with_scale(scale);
        assert_eq!(t.scale, scale);
    }

    #[test]
    fn to_mat4_identity_transform_gives_identity() {
        assert_eq!(Transform::new().to_mat4(), Mat4::IDENTITY);
    }

    #[test]
    fn to_mat4_translation_in_w_axis() {
        let t = Transform::new().with_pos(Vec3::new(5.0, 3.0, 1.0));
        let m = t.to_mat4();
        // column-major: translation sits in w_axis
        assert!((m.w_axis.x - 5.0).abs() < f32::EPSILON);
        assert!((m.w_axis.y - 3.0).abs() < f32::EPSILON);
        assert!((m.w_axis.z - 1.0).abs() < f32::EPSILON);
        assert!((m.w_axis.w - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn to_mat4_uniform_scale_on_diagonal() {
        let t = Transform::new().with_scale(Vec3::splat(2.0));
        let m = t.to_mat4();
        assert!((m.x_axis.x - 2.0).abs() < f32::EPSILON);
        assert!((m.y_axis.y - 2.0).abs() < f32::EPSILON);
        assert!((m.z_axis.z - 2.0).abs() < f32::EPSILON);
    }
}
