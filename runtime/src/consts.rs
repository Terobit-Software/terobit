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

/// direction from any lit surface toward the sun (not normalized, shader/cpu code
/// normalizes it). shared between the main shader's diffuse term and the shadow
/// cascade math, which needs the opposite (the direction the light travels), so both
/// sides always agree on where "the sun" is
pub const SUN_DIRECTION_TO_LIGHT: [f32; 3] = [0.5, 1.0, 0.3];

/// how many cascades the directional light's shadow map splits the view frustum into.
/// hardcoded to 4 on the shader side too (`LightUniform` packs splits into a vec4) -
/// change both together if this ever needs to move
pub const SHADOW_CASCADE_COUNT: usize = 4;

/// resolution of each cascade's shadow map layer. also hardcoded to match in
/// root.wgsl's `SHADOW_MAP_SIZE` constant (wgsl can't see rust consts)
pub const SHADOW_MAP_SIZE: u32 = 2048;

/// world-space distance from the camera beyond which nothing gets shadowed. the
/// farthest cascade split lands here. scene is roughly 400-600 units across, so this
/// comfortably covers it without wasting cascade resolution on empty far distance
pub const SHADOW_MAX_DISTANCE: f32 = 600.0;

/// blend factor between a uniform and logarithmic cascade split scheme (the "practical
/// split scheme"). 0 = purely uniform splits, 1 = purely logarithmic. logarithmic gives
/// near cascades much more resolution, which is where shadow aliasing is most visible,
/// at the cost of the far cascades covering more ground per texel
pub const SHADOW_SPLIT_LAMBDA: f32 = 0.6;

/// max distinct meshes gpu-culled in a single frame. cost per slot is just a small
/// uniform/indirect-args buffer entry, not per instance, so this can be generous
pub const MAX_CULL_MESH_SLOTS: u32 = 256;
