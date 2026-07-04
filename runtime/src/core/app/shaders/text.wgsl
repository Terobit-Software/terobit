@group(0) @binding(0) var<uniform> screen_size: vec2<f32>;
@group(1) @binding(0) var atlas_tex: texture_2d<f32>;
@group(1) @binding(1) var atlas_sampler: sampler;

struct VertexInput {
    @location(0) corner: vec2<f32>,
};

struct InstanceInput {
    @location(1) screen_pos: vec2<f32>,
    @location(2) size:       vec2<f32>,
    @location(3) uv_min:     vec2<f32>,
    @location(4) uv_max:     vec2<f32>,
    @location(5) color:      vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0)       uv:            vec2<f32>,
    @location(1)       color:         vec4<f32>,
};

@vertex
fn vs_main(vert: VertexInput, inst: InstanceInput) -> VertexOutput {
    let pixel_pos = inst.screen_pos + vert.corner * inst.size;
    let ndc_x = (pixel_pos.x / screen_size.x) * 2.0 - 1.0;
    let ndc_y = 1.0 - (pixel_pos.y / screen_size.y) * 2.0;

    var out: VertexOutput;
    out.clip_position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    out.uv = mix(inst.uv_min, inst.uv_max, vert.corner);
    out.color = inst.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let alpha = textureSample(atlas_tex, atlas_sampler, in.uv).r;
    if alpha < 0.05 {
        discard;
    }
    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}
