/// Depth/stencil texture format used for the depth buffer.
pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

/// Vertical field of view of the placeholder camera, in radians.
pub const CAMERA_FOV_Y_RADIANS: f32 = std::f32::consts::FRAC_PI_4;

/// Near clip plane distance for the placeholder camera.
pub const CAMERA_NEAR: f32 = 0.1;

/// Far clip plane distance for the placeholder camera.
pub const CAMERA_FAR: f32 = 100.0;

/// World-space position of the placeholder camera, looking at the origin.
pub const CAMERA_POSITION: glam::Vec3 = glam::Vec3::new(0.0, 0.0, 3.0);

/// Radians/second the demo cube spins, split evenly across the X and Y axes,
/// so there's something visibly 3D on screen before a real camera exists.
pub const CUBE_SPIN_RADIANS_PER_SECOND: f32 = 0.8;
