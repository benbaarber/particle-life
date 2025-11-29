struct Params {
    bound: vec2f,
    num_cultures: u32,
    culture_size: u32,
    num_particles: u32,
    aoe: f32,
    aoe2: f32,
    damping: f32,
    bin_size: f32,
    grid_w: u32,
}

struct Particle {
    pos: vec2f,
    vel: vec2f,
};

@group(0) @binding(0)
var<uniform> params: Params;
@group(0) @binding(1)
var<storage, read> gravity_mesh: array<f32>;
@group(0) @binding(2)
var<storage, read_write> bin_counts: array<atomic<u32>>;
@group(0) @binding(3)
var<storage, read_write> bin_ixs: array<u32>;
@group(0) @binding(4)
var<storage, read_write> bin_offsets: array<u32>;
@group(0) @binding(5)
var<storage, read_write> bin_current: array<atomic<u32>>;
@group(0) @binding(6)
var<storage, read_write> bins: array<u32>;
@group(1) @binding(0)
var<storage, read> particles: array<Particle>;
@group(1) @binding(1)
var<storage, read_write> particles_out: array<Particle>;

@compute @workgroup_size(64)
fn compute_bin_ixs_and_counts(@builtin(global_invocation_id) gid: vec3u) {
    let i = gid.x;
    if i >= params.num_particles { return; }

    // Compute ix
    let p = particles[i];
    let bx = u32(p.pos.x / params.bin_size);
    let by = u32(p.pos.y / params.bin_size);
    let bi = by * params.grid_w + bx;
    bin_ixs[i] = bi;

    // Inc count
    atomicAdd(&bin_counts[bi], 1u);
}

@compute @workgroup_size(1)
fn compute_bin_offsets() {
    let n = params.grid_w * params.grid_w;
    var sum = 0u;
    for (var i = 0u; i < n; i++) {
        bin_offsets[i] = sum;
        atomicStore(&bin_current[i], sum);
        sum += atomicLoad(&bin_counts[i]);
    }
    bin_offsets[n] = sum;
}

@compute @workgroup_size(64)
fn build_bin(@builtin(global_invocation_id) gid: vec3u) {
    let i = gid.x;
    if i >= params.num_particles { return; }
    let bi = bin_ixs[i];
    let o = atomicAdd(&bin_current[bi], 1u);
    bins[o] = i;
}

@compute @workgroup_size(64)
fn compute_force(@builtin(global_invocation_id) gid: vec3u) {
    let i = gid.x;
    if i >= params.num_particles { return; }

    let p1 = particles[i];
    let c = (i / params.culture_size) * params.num_cultures;

    let gw = i32(params.grid_w);
    let bi = i32(bin_ixs[i]);
    let bx = bi % gw;
    let by = bi / gw;

    var force = vec2f(0.0);

    for (var dy = -1i; dy <= 1; dy++) {
        for (var dx = -1i; dx <= 1; dx++) {
            let lbx = bx + dx;
            let lby = by + dy;
            if lbx < 0 || lby < 0 || lbx >= gw || lby >= gw {
                continue;
            }
            let lbi = u32(bi + dy * gw + dx);
            let bs = bin_offsets[lbi];
            let be = bin_offsets[lbi+1];

            for (var b = bs; b < be; b++) {
                let j = bins[b];
                if i == j { continue; }
                let p2 = particles[j];
                let d = p2.pos - p1.pos;
                let d2 = dot(d, d);
                if d2 < params.aoe2 {
                    let g = gravity_mesh[c + j / params.culture_size];
                    force += normalize(d) * g;
                }
            }
        }
    }

    var pos = p1.pos;
    var vel = (p1.vel + force) * params.damping;
    var bound = params.bound;

    if pos.x <= 0.0 {
        vel.x = abs(vel.x);
        pos.x = 0.0;
    } else if pos.x >= bound.x {
        vel.x = -abs(vel.x);
        pos.x = bound.x;
    }

    if pos.y <= 0.0 {
        vel.y = abs(vel.y);
        pos.y = 0.0;
    } else if pos.y >= bound.y {
        vel.y = -abs(vel.y);
        pos.y = bound.y;
    }

    pos += vel;

    particles_out[i].pos = pos;
    particles_out[i].vel = vel;
}
