// depth-only pass: renders one cascade's shadow map. no fragment stage at all, this
// pipeline only ever writes depth

struct LightMatrix {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0) var<uniform> light: LightMatrix;

struct VertexInput {
    @location(0) position: vec4<f32>,
};

// only the model matrix is needed here; tex_coords/normal/prev_model/color are still
// present in the shared vertex/instance buffers this pass reuses from the main pass,
// just unread
struct InstanceInput {
    @location(3) model_0: vec4<f32>,
    @location(4) model_1: vec4<f32>,
    @location(5) model_2: vec4<f32>,
    @location(6) model_3: vec4<f32>,
};

@vertex
fn vs_main(vert: VertexInput, inst: InstanceInput) -> @builtin(position) vec4<f32> {
    let model = mat4x4<f32>(inst.model_0, inst.model_1, inst.model_2, inst.model_3);
    return light.view_proj * (model * vert.position);
}
