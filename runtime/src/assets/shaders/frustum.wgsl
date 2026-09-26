@group(0) @binding(1) var<uniform> frustum: Frustum; // 6 planes
@group(1) @binding(0) var<storage, read> objects: array<InstanceInput>;
@group(1) @binding(1) var<storage, read_write> visible: array<u32>;
@group(1) @binding(2) var<storage, read_write> args: DrawSettings;

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
    model_col0: vec4<f32>,
    model_col1: vec4<f32>,
    model_col2: vec4<f32>,
    model_col3: vec4<f32>,
    bounding_sphere: vec4<f32>,
};

struct Frustum {
    planes: array<vec4<f32>, 6>,
};

fn sphere_in_frustum(sphere: vec4<f32>) -> bool {
    for (var i = 0u; i < 6u; i++) {
        let p = frustum.planes[i];
        if (dot(p.xyz, sphere.xyz) + p.w < -sphere.w) {
            return false;
        }
    }
    return true;
}

@compute @workgroup_size(64)
fn cull (@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;

    if (i >= arrayLength(&objects)) { return; }

    if (sphere_in_frustum(objects[i].bounding_sphere)) {
        let slot = atomicAdd(&args.instance_count, 1u);
        visible[slot] = i;
    }
}
