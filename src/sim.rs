#![allow(unused)]

use glam::{Vec2, vec2};
use macroquad::{color::Color, input::MouseButton};
use quadtree::{
    BHQuadtree, Point, Quadtree, WeightedPoint,
    shapes::{Rect, Shape},
};
use rand::Rng;
use rand_distr::{Distribution, Uniform};

use crate::util::random_color;

#[derive(Clone, Debug)]
pub struct SimConfig {
    pub bound: Rect,
    pub num_cultures: usize,
    pub culture_size: usize,
    pub aoe2: f32,
    pub theta: f32,
    pub damping: f32,
    pub cursor_aoe2: f32,
    pub cursor_force: f32,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            bound: Rect::new(Vec2::ZERO, vec2(1000.0, 800.0)),
            num_cultures: 5,
            culture_size: 5000,
            aoe2: 100.0 * 100.0,
            theta: 0.9,
            damping: 0.5,
            cursor_aoe2: 200.0 * 200.0,
            cursor_force: 400.0,
        }
    }
}

impl SimConfig {
    pub fn new(
        num_cultures: usize,
        culture_size: usize,
        aoe: f32,
        theta: f32,
        damping: f32,
        cursor_aoe: f32,
        cursor_force: f32,
    ) -> Self {
        Self {
            bound: Rect::new(Vec2::ZERO, vec2(1000.0, 800.0)),
            num_cultures,
            culture_size,
            aoe2: aoe * aoe,
            theta,
            damping,
            cursor_aoe2: cursor_aoe * cursor_aoe,
            cursor_force,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Particle {
    // pub last_pos: Vec2,
    pub pos: Vec2,
    pub vel: Vec2,
}

impl Particle {
    fn new(bound: Rect) -> Self {
        let mut rng = rand::rng();
        Self {
            pos: vec2(
                rng.random_range(0..bound.bb().x as u32) as f32,
                rng.random_range(0..bound.bb().y as u32) as f32,
            ),
            vel: Vec2::ZERO,
        }
    }

    /// Get the force another particle exerts on this particle given the gravitational constant g.
    fn _naive_force(&self, other: &Particle, g: f32, aoe: f32) -> Vec2 {
        let d = Vec2::distance(self.pos, other.pos);
        if d > 0.0 && d <= aoe {
            let dp = other.pos - self.pos;
            dp * (g / (2.0 * d))
        } else {
            Vec2::ZERO
        }
    }

    /// Get the force a weighted approximated point exerts on this particle given the gravitational constant g.
    fn force(&self, point: &WeightedPoint, g: f32, aoe2: f32) -> Vec2 {
        let d2 = Vec2::distance_squared(self.pos, point.pos);
        if d2 > 0.0 && d2 <= aoe2 {
            let dir = (point.pos - self.pos).normalize();
            dir * g * (point.mass as f32)
        } else {
            Vec2::ZERO
        }
    }

    fn cursor_force(&self, caoe2: f32, cforce: f32) -> Vec2 {
        if macroquad::input::is_mouse_button_down(MouseButton::Left) {
            // Repel on left click
            let (mx, my) = macroquad::input::mouse_position();
            let mouse = vec2(mx, my);
            let d2 = Vec2::distance_squared(mouse, self.pos);
            if d2 > 0.0 && d2 <= caoe2 {
                let dir = (mouse - self.pos).normalize();
                dir * -cforce
            } else {
                Vec2::ZERO
            }
        } else if macroquad::input::is_mouse_button_down(MouseButton::Right) {
            // Attract on right click
            let (mx, my) = macroquad::input::mouse_position();
            let mouse = vec2(mx, my);
            let d2 = Vec2::distance_squared(mouse, self.pos);
            if d2 > 0.0 && d2 <= caoe2 {
                let dir = (mouse - self.pos).normalize();
                dir * cforce
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
}

impl Culture {
    fn new(color: Color, size: usize, bound: Rect, bh_theta: f32) -> Self {
        let particles = std::iter::repeat_with(|| Particle::new(bound))
            .take(size)
            .collect::<Vec<_>>();

        Self {
            color,
            particles,
            qt: BHQuadtree::new(10, 8, bh_theta),
        }
    }

    /// Reconstruct this culture's quadtree
    fn quadtree(&mut self) {
        let items = self
            .particles
            .iter()
            .map(|p| WeightedPoint::new(p.pos, 1.0))
            .collect::<Vec<_>>();
        self.qt.build(items);
    }

    fn _naive_force(&self, other: &Culture, g: f32, aoe: f32) -> Vec<Vec2> {
        self.particles
            .iter()
            .map(|p1| {
                // Accumulate force on p1
                other
                    .particles
                    .iter()
                    .fold(Vec2::ZERO, |acc, p2| acc + p1._naive_force(p2, g, aoe))
            })
            .collect()
    }

    fn force(&self, other: &Culture, g: f32, aoe2: f32) -> Vec<Vec2> {
        self.particles
            .iter()
            .map(|p1| {
                // Accumulate force on p1
                other.qt.accumulate(p1.pos, |wp| p1.force(&wp, g, aoe2))
            })
            .collect()
    }
}

#[derive(Debug)]
pub struct World {
    conf: SimConfig,
    cultures: Vec<Culture>,
    gravity_mesh: Vec<Vec<f32>>,
    force_tensor: Vec<Vec<Vec2>>,
    cursor_force_tensor: Vec<Vec<Vec2>>,
    i: u64,
}

impl World {
    pub fn new(conf: SimConfig) -> Self {
        // Spawn cultures
        let cultures = (0..conf.num_cultures)
            .map(|_| Culture::new(random_color(), conf.culture_size, conf.bound, conf.theta))
            .collect::<Vec<_>>();

        // Generate random gravity mesh
        let mut rng = rand::rng();
        let distr = Uniform::new_inclusive(-1., 1.).unwrap();
        let gravity_mesh = (0..conf.num_cultures)
            .map(|_| {
                distr
                    .sample_iter(&mut rng)
                    .take(conf.num_cultures)
                    .collect()
            })
            .collect();

        println!(
            "Cultures: {}\nCulture size: {}\nGravity Mesh: {:?}",
            conf.num_cultures, conf.culture_size, &gravity_mesh
        );

        let force_tensor = vec![vec![Vec2::ZERO; conf.culture_size]; conf.num_cultures];
        let cursor_force_tensor = vec![vec![Vec2::ZERO; conf.culture_size]; conf.num_cultures];

        Self {
            cultures,
            gravity_mesh,
            force_tensor,
            cursor_force_tensor,
            i: 0,
            conf,
        }
    }

    pub fn step(&mut self, tau: f32) {
        // Regenerate quadtrees
        for culture in &mut self.cultures {
            culture.quadtree();
        }

        // Compute rolling slice of force tensor
        let c1 = (self.i % self.cultures.len() as u64) as usize;
        // for c1 in 0..self.cultures.len() {
        self.compute_force_tensor_slice(c1);

        for f in &mut self.force_tensor[c1] {
            *f /= self.cultures.len() as f32;
        }
        // }

        // Compute cursor force tensor
        for (c, culture) in self.cultures.iter().enumerate() {
            for (p, particle) in culture.particles.iter().enumerate() {
                self.cursor_force_tensor[c][p] =
                    particle.cursor_force(self.conf.cursor_aoe2, self.conf.cursor_force);
            }
        }

        // Apply force tensor
        let bound = self.conf.bound;
        for (c, culture) in self.cultures.iter_mut().enumerate() {
            for (p, particle) in culture.particles.iter_mut().enumerate() {
                let force = self.force_tensor[c][p] + self.cursor_force_tensor[c][p];
                particle.vel = (particle.vel + force) * self.conf.damping;
                if particle.pos.x <= 0. {
                    particle.vel.x = (particle.vel.x as f32).abs();
                    particle.pos.x = 0.;
                } else if particle.pos.x >= bound.bb().x {
                    particle.vel.x = -(particle.vel.x as f32).abs();
                    particle.pos.x = bound.bb().x;
                }
                if particle.pos.y <= 0. {
                    particle.vel.y = (particle.vel.y as f32).abs();
                    particle.pos.y = 0.;
                } else if particle.pos.y >= bound.bb().y {
                    particle.vel.y = -(particle.vel.y as f32).abs();
                    particle.pos.y = bound.bb().y;
                }
                particle.pos += particle.vel * tau;
            }
        }

        self.i += 1;
    }

    fn compute_force_tensor_slice(&mut self, c1: usize) {
        self.force_tensor[c1].fill(Vec2::ZERO);
        for c2 in 0..self.cultures.len() {
            let forces = self.cultures[c1].force(
                &self.cultures[c2],
                self.gravity_mesh[c1][c2],
                self.conf.aoe2,
            );
            for p in 0..forces.len() {
                self.force_tensor[c1][p] += forces[p];
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

    pub fn export_gravity_mesh_json(&self) -> String {
        serde_json::to_string(&self.gravity_mesh).expect("Gravity mesh is serializable")
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
