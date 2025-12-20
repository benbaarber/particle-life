struct Params {
    bound: vec2f,
    num_cultures: u32,
    culture_size: u32,
    aoe: f32,
    aoe2: f32,
    damping: f32,
}

struct VInput {
    @location(0) pos: vec2f,
}

struct VOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) color: vec4f,
    @location(1) local_pos: vec2f,
}

@group(0) @binding(0)
var<uniform> params: Params;
@group(0) @binding(1)
var<storage, read> colors: array<vec4f>;

const QUAD = array(
    vec2f(-1, -1),
    vec2f(1, -1),
    vec2f(-1, 1),
    vec2f(-1, 1),
    vec2f(1, -1),
    vec2f(1, 1),
);

@vertex
fn vs_main(
    vert: VInput,
    @builtin(instance_index) i: u32,
    @builtin(vertex_index) vi: u32,
) -> VOutput {
    let ndc = vec2f(
        vert.pos.x / params.bound.x * 2.0 - 1.0,
        vert.pos.y / params.bound.y * 2.0 - 1.0
    );
    let pos = ndc + QUAD[vi] * 0.002;
    var out: VOutput;
    out.color = colors[i / params.culture_size];
    out.clip_position = vec4(pos, 0.0, 1.0);
    out.local_pos = QUAD[vi];
    return out;
}

@fragment
fn fs_main(in: VOutput) -> @location(0) vec4f {
    // this glow doesnt actually do shit
    let dist = length(in.local_pos);
    
    // Multi-layer glow - bright core + soft halo
    let core = 1.0 - smoothstep(0.0, 0.3, dist); // Bright center
    let glow = exp(-dist * 0.1) * 2.0; // Soft outer glow
    let total = core + glow;
    return vec4f(in.color.rgb, total);
}
