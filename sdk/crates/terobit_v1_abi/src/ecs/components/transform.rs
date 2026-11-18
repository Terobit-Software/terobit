/// Describes a Host Transform Component.
///
/// Composed of a Translation (f32x3), a Rotation (f32x4), and a Scale (f32x3)
pub struct TransformComponent {
    pub translate_x: f32,
    pub translate_y: f32,
    pub translate_z: f32,
    pub rotation_a: f32,
    pub rotation_i: f32,
    pub rotation_j: f32,
    pub rotation_k: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub scale_z: f32,
}
