#![allow(unused)]

use std::time::Instant;

use particle_life::mq;

fn bench_mq(culture_size: usize, steps: usize) {
    let mut world = mq::World::new(mq::SimConfig {
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
    bench_mq(100, 100);
    bench_mq(1000, 100);
    bench_mq(5000, 100);
    bench_mq(20000, 100);
}
