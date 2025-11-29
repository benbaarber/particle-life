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
}
