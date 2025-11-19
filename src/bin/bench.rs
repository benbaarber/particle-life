use std::time::Instant;

use particle_life::mq::sim::{SimConfig, World};

fn bench(culture_size: usize, steps: usize) {
    let mut world = World::new(SimConfig {
        gpu: true,
        num_cultures: 10,
        culture_size,
        is_interactive: false,
        ..Default::default()
    });

    let start = Instant::now();
    for _ in 0..steps {
        world.step(1.0);
    }
    let avg = start.elapsed().as_nanos() / (steps as u128 * 1000);
    println!(
        "[{} particles] [{} steps] Avg time: {} ms",
        culture_size * 10,
        steps,
        avg as f64 / 1000.0
    );
}

fn main() {
    bench(100, 100);
    bench(1000, 100);
    bench(5000, 100);
    bench(20000, 100);
}
