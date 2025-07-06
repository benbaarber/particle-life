mod sim;
mod util;

use ::glam::{Vec2, vec2};
use macroquad::prelude::*;
use quadtree::shapes::Rect;
use sim::World;

const VEL_RATIO: f64 = 15.0;
const SIM_TIMESTEP: f64 = 1.0 / 60.0; // secs
const MAX_ACCUMULATOR: f64 = 1.0;

const BOUND: Rect = Rect::new(Vec2::ZERO, vec2(1000.0, 800.0));
const NUM_CULTURES: usize = 20;
const CULTURE_SIZE: usize = 2000;
const AOE: f32 = 100.0;
const THETA: f32 = 0.9;
const DAMPING: f32 = 0.5;
const CURSOR_AOE: f32 = 200.0;
const CURSOR_FORCE: f32 = 400.0;

fn window_conf() -> Conf {
    let window_bound = BOUND.bb() * 1.5;
    Conf {
        window_title: "Particle Life".to_string(),
        window_height: window_bound.y as i32,
        window_width: window_bound.x as i32,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut world = World::new(BOUND, NUM_CULTURES, CULTURE_SIZE, AOE, THETA, false);

    let tau = (VEL_RATIO * SIM_TIMESTEP) as f32;

    let mut acc = 0.0;
    let mut last_tick = get_time();

    let mut i = 0;

    loop {
        let cur_tick = get_time();
        let frame_time = cur_tick - last_tick;
        last_tick = cur_tick;
        acc += frame_time;
        acc = f64::min(acc, SIM_TIMESTEP * MAX_ACCUMULATOR);

        while acc >= SIM_TIMESTEP {
            world.step(tau);
            println!("STEP {}", i);
            i += 1;
            acc -= SIM_TIMESTEP;
        }

        world.render();

        next_frame().await;
    }
}
