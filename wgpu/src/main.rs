mod app;
mod util;

use serde::{Deserialize, Serialize};
use util::random_gravity_mesh_flat;

#[derive(Debug, Serialize, Deserialize)]
struct SimParams {
    aoe: f32,
    damping: f32,
    mesh: Vec<f32>,
}

impl SimParams {
    fn random(num_cultures: usize) -> Self {
        Self {
            aoe: rand::random_range(10.0..100.0),
            damping: rand::random_range(0.1..0.5),
            mesh: random_gravity_mesh_flat(num_cultures),
        }
    }
}

fn main() {
    let mut num_cultures = 8;
    let culture_size = 10000;
    let simp = std::env::args().skip(1).next();
    let simp = match simp {
        Some(simp) => {
            let simp: SimParams = serde_json::from_str(&simp).unwrap();
            num_cultures = simp.mesh.len().isqrt();
            simp
        }
        None => SimParams::random(num_cultures),
    };
    println!("SimParams\n{}", serde_json::to_string(&simp).unwrap());
    let params = app::GpuParams::new(num_cultures as u32, culture_size, simp.aoe, simp.damping);
    app::run(params, simp.mesh);
}
