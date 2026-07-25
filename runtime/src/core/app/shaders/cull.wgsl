// per-instance gpu culling: frustum test against this frame's view_proj. survivors get
// compacted into visible_instances and counted into the mesh's indirect draw args, all
// within that mesh's own segment of the buffers (bounded by mesh.instance_count, so it
// never spills into a neighboring mesh's range)
//
// occlusion culling (hi-z reprojected from last frame) is disabled for now: it caused
// a persistent every-other-frame flicker (whole mesh groups popping in/out) that
// survived a real bias fix and a confirmed-correct cull-data readback, so the bug is
// somewhere in the temporal reprojection approach itself, not a simple tuning issue.
// hi-z build/globals plumbing (cull_globals near/far/hiz_mip_count, the hiz texture
// and its bind group entries below) is left in place unused so re-enabling this is a
// small diff once it's done properly (most likely: two-phase hi-z, no cross-frame
// depth reuse)

struct InstanceRaw {
    model: mat4x4<f32>,
    prev_model: mat4x4<f32>,
    color: vec4<f32>,
};

struct IndirectArgs {
    index_count: u32,
    instance_count: atomic<u32>,
    first_index: u32,
    base_vertex: i32,
    first_instance: u32,
};

struct MeshCullInfo {
    base_offset: u32,
    instance_count: u32,
    local_radius: f32,
    _pad0: f32,
    local_center: vec3<f32>,
    _pad1: f32,
};

struct CullGlobals {
    frustum_planes: array<vec4<f32>, 6>,
    prev_view_proj: mat4x4<f32>,
    screen_size: vec2<f32>,
    hiz_mip_count: f32,
    near: f32,
    far: f32,
    // three plain scalars, not a vec3: wgsl forces vec3 to 16-byte alignment in the
    // uniform address space (adding a hidden gap before it and rounding the whole
    // struct up), which doesn't match rust's flat `[f32; 3]` on the other side
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
};

@group(0) @binding(0) var<storage, read> instances: array<InstanceRaw>;
@group(0) @binding(1) var<storage, read_write> visible_instances: array<InstanceRaw>;
@group(0) @binding(2) var<storage, read_write> indirect_args: IndirectArgs;
@group(0) @binding(3) var<uniform> mesh: MeshCullInfo;
@group(0) @binding(4) var<uniform> globals: CullGlobals;
@group(0) @binding(5) var hiz_tex: texture_2d<f32>;
@group(0) @binding(6) var hiz_sampler: sampler;

@compute @workgroup_size(64)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let local_index = gid.x;
    if (local_index >= mesh.instance_count) {
        return;
    }

    let read_index = mesh.base_offset + local_index;
    let inst = instances[read_index];
    let model = inst.model;

    let world_center = (model * vec4<f32>(mesh.local_center, 1.0)).xyz;
    let scale = max(length(model[0].xyz), max(length(model[1].xyz), length(model[2].xyz)));
    let world_radius = mesh.local_radius * scale;

    // frustum test: sphere outside any plane means fully outside the view
    for (var i = 0u; i < 6u; i++) {
        let p = globals.frustum_planes[i];
        if (dot(p.xyz, world_center) + p.w < -world_radius) {
            return;
        }
    }

    let write_index = atomicAdd(&indirect_args.instance_count, 1u);
    visible_instances[mesh.base_offset + write_index] = inst;
}
