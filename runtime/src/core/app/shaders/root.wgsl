@group(0) @binding(0) var<uniform> mvp: mat4x4<f32>;

struct VertexInput {
    @location(0) position:   vec4<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal:     vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0)       normal:        vec3<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = mvp * in.position;
    out.normal = normalize(in.normal);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let light_dir  = normalize(vec3<f32>(0.5, 1.0, 0.3));
    let base_color = vec3<f32>(1.0, 0.0, 0.0);

    let diffuse   = max(dot(in.normal, light_dir), 0.0);
    let ambient   = 0.15;
    let intensity = ambient + (1.0 - ambient) * diffuse;

    return vec4<f32>(base_color * intensity, 1.0);
}