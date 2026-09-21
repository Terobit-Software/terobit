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

/// Number of per-instance model matrices the instance buffer has room for
/// before it first has to grow; it doubles from here as more meshes spawn.
pub const INITIAL_INSTANCE_CAPACITY: usize = 64;
