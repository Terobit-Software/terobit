use crate::ecs::components::{Component, ComponentId};

/// Position, rotation and scale of an entity.
///
/// `#[repr(C)]` and `Pod` so it can be copied to and from host memory as raw bytes
/// (40 bytes, no padding). Rotation is a quaternion in `[x, y, z, w]` order.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Transform {
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}

impl Component for Transform {
    const ID: ComponentId = 1;
}

impl Transform {
    pub const IDENTITY: Self = Self {
        translation: [0.0, 0.0, 0.0],
        rotation: [0.0, 0.0, 0.0, 1.0],
        scale: [1.0, 1.0, 1.0],
    };

    pub fn new() -> Self {
        Self::default()
    }

    pub fn translation(mut self, translation: [f32; 3]) -> Self {
        self.translation = translation;
        self
    }

    pub fn rotation(mut self, rotation: [f32; 4]) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn scale(mut self, scale: [f32; 3]) -> Self {
        self.scale = scale;
        self
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::IDENTITY
    }
}
