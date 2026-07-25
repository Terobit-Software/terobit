// builds one hi-z mip level from the previous one by taking the max depth (farthest)
// over each 2x2 texel footprint. max (not min/avg) so a coarse mip only ever reports
// "there's an occluder at least this far away everywhere in this footprint" - the
// conservative bound cull.wgsl needs to avoid culling something that's only occluded
// in part of the footprint

@group(0) @binding(0) var src_mip: texture_storage_2d<r32float, read>;
@group(0) @binding(1) var dst_mip: texture_storage_2d<r32float, write>;

@compute @workgroup_size(8, 8)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let dst_size = textureDimensions(dst_mip);
    if (gid.x >= dst_size.x || gid.y >= dst_size.y) {
        return;
    }

    let src_size = textureDimensions(src_mip);
    let base = vec2<i32>(gid.xy) * 2;

    var max_depth = textureLoad(src_mip, clamp(base, vec2<i32>(0), vec2<i32>(src_size) - 1)).r;
    let offsets = array<vec2<i32>, 3>(vec2<i32>(1, 0), vec2<i32>(0, 1), vec2<i32>(1, 1));
    for (var i = 0u; i < 3u; i++) {
        let coord = clamp(base + offsets[i], vec2<i32>(0), vec2<i32>(src_size) - 1);
        max_depth = max(max_depth, textureLoad(src_mip, coord).r);
    }

    textureStore(dst_mip, vec2<i32>(gid.xy), vec4<f32>(max_depth, 0.0, 0.0, 0.0));
}
