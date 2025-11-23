use std::time::{Duration, Instant};

use egui_macroquad::egui::{self, Widget};
use glam::{Vec2, vec2};
use macroquad::{
    input::{KeyCode, is_key_pressed},
    miniquad,
};
use quadtree::shapes::Rect;

use super::sim::{SimConfig, World};

#[derive(Clone, Debug)]
pub struct Config {
    pub bound: Rect,
    pub num_cultures: usize,
    pub culture_size: usize,
    pub aoe: f32,
    pub theta: f32,
    pub damping: f32,
    pub cursor_aoe: f32,
    pub cursor_force: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bound: Rect::new(Vec2::ZERO, vec2(1000.0, 800.0)),
            num_cultures: 5,
            culture_size: 5000,
            aoe: 100.0,
            theta: 0.9,
            damping: 0.5,
            cursor_aoe: 200.0,
            cursor_force: 400.0,
        }
    }
}

impl Config {
    fn freeze(&self) -> SimConfig {
        SimConfig {
            bound: self.bound,
            num_cultures: self.num_cultures,
            culture_size: self.culture_size,
            aoe2: self.aoe * self.aoe,
            theta: self.theta,
            damping: self.damping,
            cursor_aoe2: self.cursor_aoe * self.cursor_aoe,
            cursor_force: self.cursor_force,
            ..Default::default()
        }
    }
}

pub struct App {
    conf: Config,
    world: World,

    // Debug
    show_fps: bool,

    // FPS
    fps: u32,
    frames: u32,
    last_tick: Instant,
}

impl App {
    pub fn new() -> Self {
        let conf = Config::default();
        let world = World::new(conf.freeze());
        Self {
            conf,
            world,
            show_fps: true,
            fps: 0,
            frames: 0,
            last_tick: Instant::now(),
        }
    }

    pub fn physics_step(&mut self, tau: f32) {
        self.world.step(tau);
        self.frames += 1;

        if self.last_tick.elapsed() >= Duration::from_secs(1) {
            self.fps = self.frames;
            self.frames = 0;
            self.last_tick = Instant::now();
        }
    }

    fn reset_world(&mut self) {
        self.world = World::new(self.conf.freeze());
    }

    fn handle_input(&mut self) {
        if is_key_pressed(KeyCode::Q) {
            miniquad::window::quit();
        }

        if is_key_pressed(KeyCode::R) {
            self.reset_world();
        }
    }

    pub fn render(&mut self) {
        use macroquad::prelude::*;

        self.world.render();

        self.handle_input();

        if self.show_fps {
            draw_text(
                &format!("{} FPS", self.fps),
                screen_width() - 40.0,
                10.0,
                12.0,
                WHITE,
            );
        }

        egui_macroquad::ui(|ctx| {
            egui::Window::new("Simulation Config")
                .default_open(false)
                .show(ctx, |ui| {
                    egui::Slider::new(&mut self.conf.num_cultures, 1..=10)
                        .text("Num Cultures")
                        .ui(ui);
                    egui::Slider::new(&mut self.conf.culture_size, 1..=10000)
                        .text("Culture Size")
                        .ui(ui);
                    egui::Slider::new(&mut self.conf.aoe, 0.0..=300.0)
                        .text("Particle AOE")
                        .ui(ui);
                    egui::Slider::new(&mut self.conf.cursor_aoe, 0.0..=300.0)
                        .text("Cursor AOE")
                        .ui(ui);
                    egui::Slider::new(&mut self.conf.cursor_force, 0.0..=500.0)
                        .text("Cursor Force")
                        .ui(ui);
                    ui.separator();
                    ui.checkbox(&mut self.show_fps, "Show FPS");
                    // ui.checkbox(&mut self.conf.gpu, "GPU");
                    ui.separator();
                    if ui.button("Run").clicked() {
                        self.reset_world();
                    }
                    if ui.button("Print gravity mesh").clicked() {
                        let mesh = self.world.export_gravity_mesh_json();
                        println!("Gravity mesh: {:?}", &mesh);
                        // miniquad::window::clipboard_set(&mesh);
                    }
                    // TODO this needs an actual clipboard crate because miniquad clipboard is not
                    // aligned with the system
                    //
                    // if ui.button("Paste gravity mesh").clicked() {
                    //     if let Some(mesh) = miniquad::window::clipboard_get() {
                    //         let mut sim_config = self.conf.freeze();
                    //         sim_config.mesh_json = Some(mesh);
                    //         self.world = World::new(sim_config);
                    //     }
                    // }
                });
        });
        egui_macroquad::draw();
    }
}
