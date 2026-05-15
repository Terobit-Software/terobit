// Bind group 0: camera — shared across all instances in the draw call
@group(0) @binding(0) var<uniform> view_proj: mat4x4<f32>;

struct VertexInput {
    @location(0) position:   vec4<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal:     vec3<f32>,
};

// Per-instance model matrix, fed from the instance vertex buffer
struct InstanceInput {
    @location(3) model_0: vec4<f32>,
    @location(4) model_1: vec4<f32>,
    @location(5) model_2: vec4<f32>,
    @location(6) model_3: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0)       normal:        vec3<f32>,
};

@vertex
fn vs_main(vert: VertexInput, inst: InstanceInput) -> VertexOutput {
    let model = mat4x4<f32>(inst.model_0, inst.model_1, inst.model_2, inst.model_3);

    var out: VertexOutput;
    out.clip_position = view_proj * model * vert.position;

    // Transform normal into world space.
    // For uniform scaling this is fine; for non-uniform you'd use the normal matrix.
    out.normal = normalize((model * vec4<f32>(vert.normal, 0.0)).xyz);

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> { // fixed signature
    let light_dir  = normalize(vec3<f32>(0.5, 1.0, 0.3));
    let base_color = vec3<f32>(1.0, 0.0, 0.0);

    let diffuse   = max(dot(in.normal, light_dir), 0.0);
    let ambient   = 0.15;
    let intensity = ambient + (1.0 - ambient) * diffuse;

    return vec4<f32>(base_color * intensity, 1.0);
}