/// default field of view in degrees
pub const DEFAULT_FOV: f32 = 45.0;

/// camera near plane distance
pub const DEFAULT_CAMERA_Z_CLOSE: f32 = 0.1;

/// camera far plane distance (note: RenderCamera hardcodes 10000.0 at runtime, this isn't used)
pub const DEFAULT_CAMERA_Z_FAR: f32 = 10.0;

/// msaa sample count. 1 = disabled
pub const MSAA_SAMPLE_COUNT: u32 = 1;