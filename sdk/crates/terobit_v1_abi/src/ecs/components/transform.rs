/// Describes a Host Transform Component.
///
/// Composed of a Translation (f32x3), a Rotation (f32x4), and a Scale (f32x3)
/// Rotation is quat, in format x, y, z, w
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Transform {
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}
