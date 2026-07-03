use glam::Vec3;

/// default field of view in degrees
pub const DEFAULT_FOV: f32 = 45.0;

/// gravity acceleration applied to all dynamic rigid bodies (m/s^2), points down along -y
pub const GRAVITY: Vec3 = Vec3::new(0.0, -9.81, 0.0);

/// upper bound on the dt fed into the physics step. keeps a stalled frame (window
/// drag, resize) from blowing up integration with a huge timestep
pub const MAX_PHYSICS_DT: f32 = 1.0 / 20.0;

/// camera near plane distance
pub const DEFAULT_CAMERA_Z_CLOSE: f32 = 0.1;

/// camera far plane distance (note: RenderCamera hardcodes 10000.0 at runtime, this isn't used)
pub const DEFAULT_CAMERA_Z_FAR: f32 = 10.0;

/// msaa sample count. 1 = disabled
pub const MSAA_SAMPLE_COUNT: u32 = 1;