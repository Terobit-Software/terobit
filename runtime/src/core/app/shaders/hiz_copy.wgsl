// copies the raw depth buffer into mip 0 of the hi-z pyramid (r32float storage texture),
// since a depth texture can't itself be bound as a storage texture for the downsample
// pass that follows

@group(0) @binding(0) var depth_tex: texture_depth_2d;
@group(0) @binding(1) var hiz_mip0: texture_storage_2d<r32float, write>;

@compute @workgroup_size(8, 8)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let size = textureDimensions(depth_tex);
    if (gid.x >= size.x || gid.y >= size.y) {
        return;
    }

    let depth = textureLoad(depth_tex, vec2<i32>(gid.xy), 0);
    textureStore(hiz_mip0, vec2<i32>(gid.xy), vec4<f32>(depth, 0.0, 0.0, 0.0));
}
