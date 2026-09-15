#[derive(bevy_ecs::component::Component)]
pub struct Transform {
    pub position: glam::Vec3,
    pub rotation: glam::Quat,
    pub scale: glam::Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: glam::Vec3::default(),
            rotation: glam::Quat::default(),
            scale: glam::Vec3::default(),
        }
    }
}

impl Transform {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn get_pos(&self) -> glam::Vec3 {
        self.position
    }

    #[inline]
    pub fn with_pos(mut self, position: glam::Vec3) -> Self {
        self.position = position;
        self
    }

    #[inline]
    pub fn set_pos(&mut self, position: glam::Vec3) {
        self.position = position;
    }

    #[inline]
    pub fn get_rot(&self) -> glam::Quat {
        self.rotation
    }

    #[inline]
    pub fn with_rot(mut self, rotation: glam::Quat) -> Self {
        self.rotation = rotation;
        self
    }

    #[inline]
    pub fn set_rot(&mut self, rotation: glam::Quat) {
        self.rotation = rotation;
    }

    #[inline]
    pub fn get_scale(&self) -> glam::Vec3 {
        self.scale
    }

    #[inline]
    pub fn with_scale(mut self, scale: glam::Vec3) -> Self {
        self.scale = scale;
        self
    }

    #[inline]
    pub fn set_scale(&mut self, scale: glam::Vec3) {
        self.scale = scale;
    }
}
