struct Params {
    bound: vec2<f32>,
    num_cultures: u32,
    culture_size: u32,
    aoe2: f32,
    damping: f32,
}

struct VInput {
    @location(0) pos: vec2<f32>,
};

struct VOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> params: Params;
@group(0) @binding(1)
var<storage, read> colors: array<vec4<f32>>;

@vertex
fn vs_main(
    vert: VInput,
    @builtin(instance_index) i: u32,
    @builtin(vertex_index) vi: u32,
) -> VOutput {
    var QUAD = array<vec2<f32>, 6>(
        vec2(-1, -1),
        vec2(1, -1),
        vec2(-1, 1),
        vec2(-1, 1),
        vec2(1, -1),
        vec2(1, 1),
    );
    let ndc = vec2<f32>(
        vert.pos.x / params.bound.x * 2.0 - 1.0,
        vert.pos.y / params.bound.y * 2.0 - 1.0
    );
    let pos = ndc + QUAD[vi] * 0.002;
    var out: VOutput;
    out.color = colors[i / params.culture_size];
    out.clip_position = vec4<f32>(pos, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(in: VOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color);
}
