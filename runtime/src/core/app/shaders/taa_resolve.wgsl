@group(0) @binding(0) var scene_tex:    texture_2d<f32>;
@group(0) @binding(1) var history_tex:  texture_2d<f32>;
@group(0) @binding(2) var velocity_tex: texture_2d<f32>;
@group(0) @binding(3) var samp:         sampler;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

// fullscreen triangle from just the vertex index, no vertex/index buffers needed
@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> VertexOutput {
    let x = f32((index << 1u) & 2u);
    let y = f32(index & 2u);

    var out: VertexOutput;
    out.uv = vec2<f32>(x, y);
    out.clip_position = vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
    return out;
}

struct FragmentOutput {
    @location(0) resolved:    vec4<f32>,
    @location(1) next_history: vec4<f32>,
};

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    let dims  = vec2<f32>(textureDimensions(scene_tex));
    let texel = 1.0 / dims;

    let current = textureSample(scene_tex, samp, in.uv);

    // 3x3 neighborhood clamp: bounds how far the history is allowed to pull the result
    // toward last frame's color, so a bad/disoccluded reprojection can't ghost forever
    var color_min = current.rgb;
    var color_max = current.rgb;
    for (var dy = -1; dy <= 1; dy++) {
        for (var dx = -1; dx <= 1; dx++) {
            let neighbor = textureSample(scene_tex, samp, in.uv + vec2<f32>(f32(dx), f32(dy)) * texel).rgb;
            color_min = min(color_min, neighbor);
            color_max = max(color_max, neighbor);
        }
    }

    let velocity   = textureSample(velocity_tex, samp, in.uv).xy;
    let history_uv = in.uv - velocity;

    var resolved: vec3<f32>;
    if (history_uv.x < 0.0 || history_uv.x > 1.0 || history_uv.y < 0.0 || history_uv.y > 1.0) {
        // reprojected off-screen (e.g. camera just panned that way), nothing to blend with
        resolved = current.rgb;
    } else {
        let history = clamp(textureSample(history_tex, samp, history_uv).rgb, color_min, color_max);

        // lean less on history the faster this pixel is moving on screen (in pixels,
        // not uv fractions, so the falloff feels the same at any resolution). a fixed
        // blend weight looks great on a still pixel but drags a visible blur behind
        // anything moving fast, camera pans especially since then every pixel on
        // screen is "moving" at once instead of just the handful near a falling object
        let pixel_speed = length(velocity * dims);
        let blend_factor = clamp(0.95 - pixel_speed * 0.05, 0.15, 0.95);
        resolved = mix(current.rgb, history, blend_factor);
    }

    var out: FragmentOutput;
    out.resolved = vec4<f32>(resolved, current.a);
    out.next_history = out.resolved;
    return out;
}
