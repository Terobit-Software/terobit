struct CameraUniform {
    view_proj:      mat4x4<f32>,
    prev_view_proj: mat4x4<f32>,
    view:           mat4x4<f32>,
    jitter:         vec2<f32>,
};

@group(0) @binding(0) var<uniform> camera: CameraUniform;

// hardcoded to 4 cascades, matching consts::SHADOW_CASCADE_COUNT (wgsl can't see rust
// consts, so cascade_splits is packed into a vec4 instead of an array to keep both
// sides honest about there being exactly 4 of them)
struct LightUniform {
    cascade_view_proj: array<mat4x4<f32>, 4>,
    cascade_splits:    vec4<f32>,
    light_dir:         vec3<f32>,
};
@group(1) @binding(0) var<uniform> light: LightUniform;

@group(2) @binding(0) var shadow_map:     texture_depth_2d_array;
@group(2) @binding(1) var shadow_sampler: sampler_comparison;

// keep in sync with consts::SHADOW_MAP_SIZE
const SHADOW_MAP_SIZE: f32 = 2048.0;

struct VertexInput {
    @location(0) position:   vec4<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal:     vec3<f32>,
};

struct InstanceInput {
    @location(3)  model_0:      vec4<f32>,
    @location(4)  model_1:      vec4<f32>,
    @location(5)  model_2:      vec4<f32>,
    @location(6)  model_3:      vec4<f32>,
    @location(7)  prev_model_0: vec4<f32>,
    @location(8)  prev_model_1: vec4<f32>,
    @location(9)  prev_model_2: vec4<f32>,
    @location(10) prev_model_3: vec4<f32>,
    @location(11) color:        vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) normal:         vec3<f32>,
    @location(1) color:          vec4<f32>,
    // unjittered clip positions this frame and last, carried through so the fragment
    // shader can turn them into a per-pixel motion vector for taa
    @location(2) current_clip:   vec4<f32>,
    @location(3) previous_clip:  vec4<f32>,
    @location(4) world_position: vec3<f32>,
};

@vertex
fn vs_main(vert: VertexInput, inst: InstanceInput) -> VertexOutput {
    let model = mat4x4<f32>(inst.model_0, inst.model_1, inst.model_2, inst.model_3);
    let prev_model = mat4x4<f32>(inst.prev_model_0, inst.prev_model_1, inst.prev_model_2, inst.prev_model_3);

    let world_pos = model * vert.position;
    let current_clip = camera.view_proj * world_pos;
    let previous_clip = camera.prev_view_proj * (prev_model * vert.position);

    var out: VertexOutput;
    out.world_position = world_pos.xyz;
    out.current_clip = current_clip;
    out.previous_clip = previous_clip;

    // jitter only the rasterized position, never the motion-vector math above,
    // otherwise the jitter itself would show up as noise in the velocity
    var jittered = current_clip;
    jittered.x += camera.jitter.x * jittered.w;
    jittered.y += camera.jitter.y * jittered.w;
    out.clip_position = jittered;

    out.normal = normalize((model * vec4<f32>(vert.normal, 0.0)).xyz);
    out.color  = inst.color;
    return out;
}

// 3x3 pcf shadow lookup for the given cascade. returns 1.0 = fully lit, 0.0 = fully
// shadowed, in between for penumbra edges from the filtering
fn shadow_factor(cascade_index: u32, world_position: vec3<f32>) -> f32 {
    let light_clip = light.cascade_view_proj[cascade_index] * vec4<f32>(world_position, 1.0);
    let ndc = light_clip.xyz / light_clip.w;
    let shadow_uv = vec2<f32>(ndc.x * 0.5 + 0.5, 1.0 - (ndc.y * 0.5 + 0.5));

    if (shadow_uv.x < 0.0 || shadow_uv.x > 1.0 || shadow_uv.y < 0.0 || shadow_uv.y > 1.0
        || ndc.z < 0.0 || ndc.z > 1.0) {
        // outside this cascade's coverage (shouldn't normally happen, the cascade was
        // sized to the camera frustum slice it's paired with): treat as lit rather
        // than guessing
        return 1.0;
    }

    let texel = 1.0 / SHADOW_MAP_SIZE;
    var sum = 0.0;
    for (var y = -1; y <= 1; y++) {
        for (var x = -1; x <= 1; x++) {
            let offset = vec2<f32>(f32(x), f32(y)) * texel;
            sum += textureSampleCompare(shadow_map, shadow_sampler, shadow_uv + offset, cascade_index, ndc.z);
        }
    }
    return sum / 9.0;
}

struct FragmentOutput {
    @location(0) color:    vec4<f32>,
    @location(1) velocity: vec2<f32>,
};

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    let diffuse = max(dot(in.normal, light.light_dir), 0.0);
    let ambient = 0.15;

    // view-space depth (distance along the camera's forward axis, not straight-line
    // distance) is what the cascade splits are defined against
    let view_space = camera.view * vec4<f32>(in.world_position, 1.0);
    let view_depth = -view_space.z;

    var shadow = 1.0;
    if (view_depth < light.cascade_splits.x) {
        shadow = shadow_factor(0u, in.world_position);
    } else if (view_depth < light.cascade_splits.y) {
        shadow = shadow_factor(1u, in.world_position);
    } else if (view_depth < light.cascade_splits.z) {
        shadow = shadow_factor(2u, in.world_position);
    } else if (view_depth < light.cascade_splits.w) {
        shadow = shadow_factor(3u, in.world_position);
    }
    // beyond the last split: past the shadow's draw distance, treat as fully lit

    let intensity = ambient + (1.0 - ambient) * diffuse * shadow;

    let current_ndc  = in.current_clip.xy / in.current_clip.w;
    let previous_ndc = in.previous_clip.xy / in.previous_clip.w;
    // ndc [-1,1] -> uv [0,1], flipping y to match texture space
    let current_uv  = vec2<f32>(current_ndc.x, -current_ndc.y) * 0.5 + 0.5;
    let previous_uv = vec2<f32>(previous_ndc.x, -previous_ndc.y) * 0.5 + 0.5;

    var out: FragmentOutput;
    out.color = vec4<f32>(in.color.rgb * intensity, in.color.a);
    out.velocity = current_uv - previous_uv;
    return out;
}
