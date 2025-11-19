use ::glam::{Vec2, vec2};
use macroquad::prelude::*;
use particle_life::mq::app::App;
use quadtree::shapes::Rect;

const VEL_RATIO: f64 = 15.0;
const SIM_TIMESTEP: f64 = 1.0 / 60.0; // secs
const MAX_ACCUMULATOR: f64 = 1.0;

const BOUND: Rect = Rect::new(Vec2::ZERO, vec2(1000.0, 800.0));

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
    let mut app = App::new();

    let tau = (VEL_RATIO * SIM_TIMESTEP) as f32;

    let mut acc = 0.0;
    let mut last_tick = get_time();

    loop {
        let cur_tick = get_time();
        let frame_time = cur_tick - last_tick;
        last_tick = cur_tick;
        acc += frame_time;
        acc = f64::min(acc, SIM_TIMESTEP * MAX_ACCUMULATOR);

        while acc >= SIM_TIMESTEP {
            app.physics_step(tau);
            acc -= SIM_TIMESTEP;
        }

        app.render();

        next_frame().await;
    }
}
