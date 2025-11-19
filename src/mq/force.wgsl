struct Params {
    num_cultures: u32,
    culture_size: u32,
    theta2: f32,
    aoe2: f32,
}

@group(0) @binding(0)
var<storage, read> particles: array<vec2<f32>>;
@group(0) @binding(1)
var<storage, read_write> forces: array<vec2<f32>>;
@group(0) @binding(2)
var<uniform> params: Params;
@group(0) @binding(3)
var<storage, read> gravity_mesh: array<f32>;

@compute
@workgroup_size(64)
fn main(
    @builtin(global_invocation_id) gid: vec3<u32>
) {
    let i = gid.x;
    let n = arrayLength(&particles);

    if i >= n { return; }

    let p1 = particles[i];
    let c = (i / params.culture_size) * params.num_cultures;
    var force = vec2(0.0);

    for (var j = 0u; j < n; j++) {
        if i == j { continue; }
        let p2 = particles[j];
        let d = p2 - p1;
        let d2 = dot(d, d);
        if d2 > 0.0 && d2 < params.aoe2 {
            let g = gravity_mesh[c + j / params.culture_size];
            force += normalize(d) * g;
        }
    }

    forces[i] = force;
}


