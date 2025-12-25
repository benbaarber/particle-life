mod app;
mod util;

use clap::Parser;
use serde::{Deserialize, Serialize};
use util::random_gravity_mesh_flat;

#[derive(Parser)]
struct Args {
    /// Sim Params json string
    simp: Option<String>,
    #[arg(short, long, default_value_t = 10)]
    cultures: u32,
    #[arg(short, long, default_value_t = 5000)]
    particles: u32,
    #[arg(short, long, default_value_t = 50.0)]
    aoe: f32,
    #[arg(short, long, default_value_t = 0.1)]
    damping: f32,
}

#[derive(Debug, Serialize, Deserialize)]
struct SimParams {
    num_cultures: u32,
    culture_size: u32,
    aoe: f32,
    damping: f32,
    mesh: Vec<f32>,
}

fn main() {
    let args = Args::parse();
    let simp = match args.simp {
        Some(s) => serde_json::from_str(&s).expect("Simp arg should be valid json"),
        None => SimParams {
            num_cultures: args.cultures,
            culture_size: args.particles,
            aoe: args.aoe,
            damping: args.damping,
            mesh: random_gravity_mesh_flat(args.cultures as usize),
        },
    };
    println!("SimParams\n{}", serde_json::to_string(&simp).unwrap());
    let params = app::GpuParams::new(simp.num_cultures, simp.culture_size, simp.aoe, simp.damping);
    app::run(params, simp.mesh);
}
