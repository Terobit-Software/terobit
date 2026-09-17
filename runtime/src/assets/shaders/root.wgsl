struct Camera {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: Camera;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
};

struct InstanceInput {
    @location(2) model_col0: vec4<f32>,
    @location(3) model_col1: vec4<f32>,
    @location(4) model_col2: vec4<f32>,
    @location(5) model_col3: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) normal: vec3<f32>,
};

@vertex
fn vs_main(input: VertexInput, instance: InstanceInput) -> VertexOutput {
    let model = mat4x4<f32>(
        instance.model_col0,
        instance.model_col1,
        instance.model_col2,
        instance.model_col3,
    );
    let normal_matrix = mat3x3<f32>(model[0].xyz, model[1].xyz, model[2].xyz);

    var out: VertexOutput;
    out.clip_position = camera.view_proj * model * vec4<f32>(input.position, 1.0);
    out.normal = normal_matrix * input.normal;
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
