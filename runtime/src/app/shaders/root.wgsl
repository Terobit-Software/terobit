struct Uniforms {
    mvp: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) normal: vec3<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = uniforms.mvp * vec4<f32>(input.position, 1.0);
    out.normal = input.normal;
    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let light_dir = normalize(vec3<f32>(0.5, 0.7, 1.0));
    let diffuse = max(dot(normalize(input.normal), light_dir), 0.0);
    let base_color = vec3<f32>(0.9, 0.15, 0.15);
    let color = base_color * (0.25 + 0.75 * diffuse);
    return vec4<f32>(color, 1.0);
}
