use crate::ecs::components::Component;

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

impl Transform {
    /// The component ID used in queries, e.g. `Read<{ Transform::ID }>`.
    pub const ID: Component = 1;

    pub const IDENTITY: Self = Self {
        translation: [0.0, 0.0, 0.0],
        rotation: [0.0, 0.0, 0.0, 1.0],
        scale: [1.0, 1.0, 1.0],
    };
}

impl Default for Transform {
    fn default() -> Self {
        Self::IDENTITY
    }
}
