#![allow(unused)]

use glam::{Vec2, vec2};
use macroquad::{color::Color, input::MouseButton};
use quadtree::{
    BHQuadtree, Point, Quadtree, WeightedPoint,
    shapes::{Rect, Shape},
};
use rand::Rng;
use rand_distr::{Distribution, Uniform};

use crate::{CURSOR_AOE, CURSOR_FORCE, DAMPING, THETA, util::random_color};

#[derive(Clone, Copy, Debug)]
pub struct Particle {
    pub pos: Vec2,
    pub vel: Vec2,
    pub aoe: f32,
}

impl Particle {
    fn new(bound: Rect, aoe: f32) -> Self {
        let mut rng = rand::rng();
        Self {
            pos: vec2(
                rng.random_range(0..bound.bb().x as u32) as f32,
                rng.random_range(0..bound.bb().y as u32) as f32,
            ),
            vel: Vec2::ZERO,
            aoe,
        }
    }

    /// Get the force another particle exerts on this particle given the gravitational constant g.
    fn _naive_force(&self, other: &Particle, g: f32) -> Vec2 {
        let d = Vec2::distance(self.pos, other.pos);
        if d > 0.0 && d <= self.aoe {
            let dp = other.pos - self.pos;
            dp * (g / (2.0 * d))
        } else {
            Vec2::ZERO
        }
    }

    /// Get the force a weighted approximated point exerts on this particle given the gravitational constant g.
    fn force(&self, point: &WeightedPoint, g: f32) -> Vec2 {
        let d = Vec2::distance(self.pos, point.pos);
        if d > 0.0 && d <= self.aoe {
            let dp = (point.pos - self.pos) / d;
            dp * g * (point.mass as f32)
        } else {
            Vec2::ZERO
        }
    }

    fn cursor_force(&self) -> Vec2 {
        if macroquad::input::is_mouse_button_down(MouseButton::Left) {
            // Repel on left click
            let (mx, my) = macroquad::input::mouse_position();
            let mouse = vec2(mx, my);
            let d = Vec2::distance(mouse, self.pos);
            if d > 0.0 && d <= CURSOR_AOE {
                let dp = mouse - self.pos;
                dp * (-CURSOR_FORCE / (2.0 * d))
            } else {
                Vec2::ZERO
            }
        } else if macroquad::input::is_mouse_button_down(MouseButton::Right) {
            // Attract on right click
            let (mx, my) = macroquad::input::mouse_position();
            let mouse = vec2(mx, my);
            let d = Vec2::distance(mouse, self.pos);
            if d > 0.0 && d <= CURSOR_AOE {
                let dp = mouse - self.pos;
                dp * (CURSOR_FORCE / (2.0 * d))
            } else {
                Vec2::ZERO
            }
        } else {
            Vec2::ZERO
        }
    }
}

impl Point for Particle {
    fn point(&self) -> Vec2 {
        self.pos
    }
}

#[derive(Debug)]
struct Culture {
    color: Color,
    particles: Vec<Particle>,
    qt: BHQuadtree,
    bound: Rect,
}

impl Culture {
    fn new(color: Color, bound: Rect, population: usize, particle_aoe: f32) -> Self {
        let particles = std::iter::repeat_with(|| Particle::new(bound, particle_aoe))
            .take(population)
            .collect::<Vec<_>>();

        Self {
            color,
            particles,
            qt: BHQuadtree::new(THETA),
            bound,
        }
    }

    /// Reconstruct this culture's quadtree
    fn quadtree(&mut self) {
        let items = self
            .particles
            .iter()
            .map(|p| WeightedPoint::new(p.pos, 1.0))
            .collect::<Vec<_>>();
        self.qt.build(items, 10);
    }

    fn _naive_force(&self, other: &Culture, g: f32) -> Vec<Vec2> {
        self.particles
            .iter()
            .map(|p1| {
                // Accumulate force on p1
                other
                    .particles
                    .iter()
                    .fold(Vec2::ZERO, |acc, p2| acc + p1._naive_force(p2, g))
            })
            .collect()
    }

    fn force(&self, other: &Culture, g: f32) -> Vec<Vec2> {
        self.particles
            .iter()
            .map(|p1| {
                // Accumulate force on p1
                other.qt.accumulate(p1.pos, |wp| p1.force(&wp, g))
            })
            .collect()
    }
}

#[derive(Debug)]
pub struct WorldConfig {
    bound: Rect,
    theta: f32,
    show_qts: bool,
}

#[derive(Debug)]
pub struct World {
    cultures: Vec<Culture>,
    gravity_mesh: Vec<Vec<f32>>,
    config: WorldConfig,
}

impl World {
    pub fn new(
        bound: Rect,
        num_cultures: usize,
        culture_size: usize,
        particle_aoe: f32,
        theta: f32,
        show_qts: bool,
    ) -> Self {
        // Set config
        let config = WorldConfig {
            bound,
            theta,
            show_qts,
        };

        // Birth cultures
        let cultures = (0..num_cultures)
            .map(|color| Culture::new(random_color(), bound, culture_size, particle_aoe))
            .collect::<Vec<_>>();

        // Generate random gravity mesh
        let mut rng = rand::rng();
        let distr = Uniform::new_inclusive(-1., 1.).unwrap();
        let gravity_mesh = (0..num_cultures)
            .map(|_| distr.sample_iter(&mut rng).take(num_cultures).collect())
            .collect();

        println!(
            "Cultures: {}\nCulture size: {}\nGravity Mesh: {:?}",
            num_cultures, culture_size, &gravity_mesh
        );

        Self {
            cultures,
            gravity_mesh,
            config,
        }
    }

    pub fn step(&mut self, tau: f32) {
        // Regenerate quadtrees
        for culture in &mut self.cultures {
            culture.quadtree();
        }

        // Generate force tensor
        let force_tensor: Vec<Vec<Vec2>> = self
            .cultures
            .iter()
            .enumerate()
            .map(|(i, c1)| {
                let mut total_forces = vec![Vec2::ZERO; c1.particles.len()];
                let cursor_forces = c1
                    .particles
                    .iter()
                    .map(|p| p.cursor_force())
                    .collect::<Vec<_>>();
                total_forces =
                    self.cultures
                        .iter()
                        .enumerate()
                        .fold(total_forces, |acc, (j, c2)| {
                            let forces = c1.force(c2, self.gravity_mesh[i][j]);
                            acc.into_iter()
                                .zip(forces)
                                .map(|(f1, f2)| f1 + f2)
                                .collect()
                        });
                for i in 0..c1.particles.len() {
                    total_forces[i] += cursor_forces[i];
                }
                total_forces
            })
            .collect();

        // Apply force tensor
        let bound = self.config.bound;
        for (i, culture) in self.cultures.iter_mut().enumerate() {
            for (j, p) in culture.particles.iter_mut().enumerate() {
                let force = force_tensor[i][j];
                p.vel = (p.vel + force) * DAMPING;
                if p.pos.x <= 0. {
                    p.vel.x = (p.vel.x as f32).abs();
                    p.pos.x = 0.;
                } else if p.pos.x >= bound.bb().x {
                    p.vel.x = -(p.vel.x as f32).abs();
                    p.pos.x = bound.bb().x;
                }
                if p.pos.y <= 0. {
                    p.vel.y = (p.vel.y as f32).abs();
                    p.pos.y = 0.;
                } else if p.pos.y >= bound.bb().y {
                    p.vel.y = -(p.vel.y as f32).abs();
                    p.pos.y = bound.bb().y;
                }
                p.pos += p.vel * tau;
            }
        }
    }

    pub fn render(&self) {
        use macroquad::prelude::*;

        clear_background(BLACK);

        for culture in &self.cultures {
            let color = culture.color;
            for p in &culture.particles {
                draw_rectangle(p.pos.x, p.pos.y, 2.0, 2.0, color);
            }
        }

        // if self.config.show_qts {
        //     for culture in &self.cultures {
        //         let color = culture.color;
        //         let qt = culture.qt;
        //         qt.query_ref_filter(&self.config.bound, |_| draw_rectangle_lines())
        //     }
        // }
    }

    // Found out WASM does not support multithreading after writing this lol
    // pub fn step_concurrent(&mut self) {
    //     let cultures = Arc::new(self.cultures.clone());
    //     let gravity_mesh = Arc::new(self.gravity_mesh.clone());
    //     let handles = (0..self.cultures.len()).map(|i| {
    //         let cultures = Arc::clone(&cultures);
    //         let gravity_mesh = Arc::clone(&gravity_mesh);
    //         thread::spawn(move || {
    //             let c1 = &cultures[i];
    //             let initial_forces = vec![na::vector![0., 0.]; c1.particles.len()];
    //             cultures.iter().enumerate().fold(initial_forces, |acc, (j, c2)| {
    //                 let forces = c1.force(c2, gravity_mesh[i][j]);
    //                 acc.into_iter()
    //                     .zip(forces)
    //                     .map(|(f1, f2)| f1 + f2)
    //                     .collect()
    //             })
    //         })
    //     });
    //     let force_tensor = handles.map(|h| h.join().unwrap()).collect::<Vec<_>>();
    //     // Apply force tensor
    //     for (i, culture) in self.cultures.iter_mut().enumerate() {
    //         for (j, p) in culture.particles.iter_mut().enumerate() {
    //             let force = force_tensor[i][j];
    //             p.vel = (p.vel + force) * 0.5;
    //             if p.pos.x <= 0. {
    //                 p.vel.x = (p.vel.x as f32).abs();
    //             } else if p.pos.x >= self.world_size as f32 {
    //                 p.vel.x = -(p.vel.x as f32).abs();
    //             }
    //             if p.pos.y <= 0. {
    //                 p.vel.y = (p.vel.y as f32).abs();
    //             } else if p.pos.y >= self.world_size as f32 {
    //                 p.vel.y = -(p.vel.y as f32).abs();
    //             }
    //             p.pos += p.vel;
    //         }
    //     }
    //     // Render on HTML Canvas
    //     self.cx.clear_rect(
    //         0.,
    //         0.,
    //         self.world_size as f32 * 2.,
    //         self.world_size as f32 * 2.,
    //     );
    //     for Culture { color, particles } in &*self.cultures {
    //         self.cx.set_fill_style(&JsValue::from_str(&color));
    //         for Particle { pos, .. } in particles {
    //             self.cx.fill_rect(pos.x, pos.y, 5., 5.);
    //         }
    //     }
    // }
}
