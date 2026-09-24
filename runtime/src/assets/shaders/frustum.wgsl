@group(0) @binding(1) var<storage, read> objects: array<InstanceInput>;
@group(0) @binding(2) var<storage, read> visible: array<u32>;
@group(0) @binding(3) var<storage, read_write> args: DrawSettings;
@group(0) @binding(4) var<uniform> frustum: Frustum; // 6 planes

struct Camera {
    view_proj: mat4x4<f32>,
};

struct DrawSettings {
    index_count: u32,
    instance_count: atomic<u32>,
    first_index: u32,
    base_vertex: i32,
    first_instance: u32,
};

struct InstanceInput {
    @location(2) model_col0: vec4<f32>,
    @location(3) model_col1: vec4<f32>,
    @location(4) model_col2: vec4<f32>,
    @location(5) model_col3: vec4<f32>,
};

struct Frustum {
    planes: array<vec4<f32>, 6>,
};

fn sphere_in_frustum(center: vec3<f32>, radius: f32) -> bool {
    for (var i = 0u; i < 6u; i++) {
        let p = frustum.planes[i];
        // Sphere is entirely behind this plane, so it's outside.
        if (dot(p.xyz, center) + p.w < -radius) {
            return false;
        }
    }
    return true;
}

@compute @workgroup_size(64)
fn frustum_cull () {

}
