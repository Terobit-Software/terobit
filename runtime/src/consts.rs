/// default field of view in degrees
pub const DEFAULT_FOV: f32 = 45.0;

/// upper bound on the dt fed into the physics step. keeps a stalled frame (window
/// drag, resize) from blowing up integration with a huge timestep
pub const MAX_PHYSICS_DT: f32 = 1.0 / 20.0;

/// max simultaneous rigid bodies jolt will track. bump this if the scene needs more
pub const MAX_PHYSICS_BODIES: u32 = 65536;

/// scratch memory jolt's temp allocator gets per physics step (bytes). needs to
/// scale with body count, jolt's own hello world example gets away with 10mb
/// for ~1000 bodies but that's nowhere near enough once you're pushing tens
/// of thousands
pub const PHYSICS_TEMP_ALLOCATOR_BYTES: u32 = 128 * 1024 * 1024;

/// camera near plane distance
pub const DEFAULT_CAMERA_Z_CLOSE: f32 = 0.1;

/// camera far plane distance (note: RenderCamera hardcodes 10000.0 at runtime, this isn't used)
pub const DEFAULT_CAMERA_Z_FAR: f32 = 10.0;

/// msaa sample count. 1 = disabled
pub const MSAA_SAMPLE_COUNT: u32 = 1;

/// max glyph quads the profiler hud can draw in one frame. trims excess rather
/// than growing the buffer every frame
pub const MAX_HUD_GLYPHS: u64 = 8192;