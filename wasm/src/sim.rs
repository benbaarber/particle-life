#![allow(unused)]

use crate::qt::QuadTree;
use na::{Normed, Point2, Vector2};
use nalgebra as na;
use rand::{
    distributions::{Distribution, Uniform},
    Rng,
};
use serde::{ser::SerializeSeq, Serialize};
use wasm_bindgen::{prelude::wasm_bindgen, JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

#[derive(Debug)]
struct Particle {
    pos: Point2<f64>,
    vel: Vector2<f64>,
    aoe: f64,
}

impl Particle {
    fn new(world: Point2<f64>, aoe: f64) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            pos: na::point![
                rng.gen_range(0..world.x as u32) as f64,
                rng.gen_range(0..world.y as u32) as f64,
            ],
            vel: Vector2::zeros(),
            aoe,
        }
    }

    /// Get the force another particle exerts on this particle given the gravitational constant g.
    fn force(&self, other: &Particle, g: f64) -> Vector2<f64> {
        let d = na::distance(&self.pos, &other.pos);
        if d > 0. && d < self.aoe {
            let dp = other.pos - self.pos;
            dp * (g / (2. * d))
        } else {
            Vector2::zeros()
        }
    }
}

impl Serialize for Particle {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.pos.len()))?;
        for e in self.pos.iter() {
            seq.serialize_element(e)?;
        }
        seq.end()
    }
}

#[derive(Debug, Serialize)]
struct Culture {
    color: String,
    particles: Vec<Particle>,
    world: Point2<f64>,
}

impl Culture {
    fn new(color: String, world: Point2<f64>, population: usize, particle_aoe: f64) -> Self {
        let particles = std::iter::repeat_with(|| Particle::new(world, particle_aoe))
            .take(population)
            .collect::<Vec<_>>();

        Self {
            color,
            particles,
            world,
        }
    }

    fn force(&self, other: &Culture, g: f64) -> Vec<Vector2<f64>> {
        let mut qt = QuadTree::new(self.world);
        for p in &self.particles {
            // TODO: Handle false
            qt.insert(&p.pos, 0);
        }

        qt.com();

        self.particles
            .iter()
            .map(|p1| {
                // Accumulate force on p1
                other
                    .particles
                    .iter()
                    .fold(Vector2::zeros(), |acc, p2| acc + p1.force(p2, g))
            })
            .collect()
    }
}

#[derive(Debug, Serialize)]
#[wasm_bindgen]
pub struct PetriDish {
    width: f64,
    height: f64,
    cultures: Vec<Culture>,
    gravity_mesh: Vec<Vec<f64>>,
    #[serde(skip)]
    cx: CanvasRenderingContext2d,
}

#[wasm_bindgen]
impl PetriDish {
    #[wasm_bindgen(constructor)]
    pub fn new(
        colors: Vec<String>,
        width: f64,
        height: f64,
        population: usize,
        particle_aoe: f64,
    ) -> Self {
        // Set panic hook
        crate::utils::set_panic_hook();

        // Birth cultures
        let cultures = colors
            .into_iter()
            .map(|color| Culture::new(color, na::point![width, height], population, particle_aoe))
            .collect::<Vec<_>>();

        // Generate random gravity mesh
        let num_cultures = cultures.len();
        let mut rng = rand::thread_rng();
        let distr = Uniform::new_inclusive(-1., 1.);
        let gravity_mesh = (0..num_cultures)
            .map(|_| distr.sample_iter(&mut rng).take(num_cultures).collect())
            .collect();

        // Bind to HTML Canvas
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id("canvas").unwrap();
        let canvas: HtmlCanvasElement = canvas
            .dyn_into::<HtmlCanvasElement>()
            .map_err(|_| ())
            .unwrap();

        let cx = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();

        Self {
            width,
            height,
            cultures,
            gravity_mesh,
            cx,
        }
    }

    pub fn step(&mut self) {
        // Generate force tensor
        let force_tensor: Vec<Vec<Vector2<f64>>> = self
            .cultures
            .iter()
            .enumerate()
            .map(|(i, c1)| {
                let initial_forces = vec![Vector2::zeros(); c1.particles.len()];
                self.cultures
                    .iter()
                    .enumerate()
                    .fold(initial_forces, |acc, (j, c2)| {
                        let forces = c1.force(c2, self.gravity_mesh[i][j]);
                        acc.into_iter()
                            .zip(forces)
                            .map(|(f1, f2)| f1 + f2)
                            .collect()
                    })
            })
            .collect();

        // Apply force tensor
        for (i, culture) in self.cultures.iter_mut().enumerate() {
            for (j, p) in culture.particles.iter_mut().enumerate() {
                let force = force_tensor[i][j];
                p.vel = (p.vel + force) * 0.5;
                if p.pos.x <= 0. {
                    p.vel.x = (p.vel.x as f64).abs();
                } else if p.pos.x >= self.width as f64 {
                    p.vel.x = -(p.vel.x as f64).abs();
                }
                if p.pos.y <= 0. {
                    p.vel.y = (p.vel.y as f64).abs();
                } else if p.pos.y >= self.height as f64 {
                    p.vel.y = -(p.vel.y as f64).abs();
                }
                p.pos += p.vel;
            }
        }

        // Render on HTML Canvas
        self.cx
            .clear_rect(0., 0., self.width as f64 * 2., self.height as f64 * 2.);
        for Culture {
            color, particles, ..
        } in &self.cultures
        {
            self.cx.set_fill_style(&JsValue::from_str(color));
            for Particle { pos, .. } in particles {
                self.cx.fill_rect(pos.x, pos.y, 5., 5.);
            }
        }
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
    //                 p.vel.x = (p.vel.x as f64).abs();
    //             } else if p.pos.x >= self.world_size as f64 {
    //                 p.vel.x = -(p.vel.x as f64).abs();
    //             }
    //             if p.pos.y <= 0. {
    //                 p.vel.y = (p.vel.y as f64).abs();
    //             } else if p.pos.y >= self.world_size as f64 {
    //                 p.vel.y = -(p.vel.y as f64).abs();
    //             }
    //             p.pos += p.vel;
    //         }
    //     }
    //     // Render on HTML Canvas
    //     self.cx.clear_rect(
    //         0.,
    //         0.,
    //         self.world_size as f64 * 2.,
    //         self.world_size as f64 * 2.,
    //     );
    //     for Culture { color, particles } in &*self.cultures {
    //         self.cx.set_fill_style(&JsValue::from_str(&color));
    //         for Particle { pos, .. } in particles {
    //             self.cx.fill_rect(pos.x, pos.y, 5., 5.);
    //         }
    //     }
    // }

    pub fn cultures(&self) -> String {
        serde_json::to_string(&*self.cultures).unwrap()
    }

    pub fn gravity_mesh(&self) -> String {
        serde_json::to_string(&*self.gravity_mesh).unwrap()
    }
}

// #[test]
// fn test() {
//     let a = na::point![1., 2.];
//     let b = na::point![2., 5.];
//     let c = na::point![3., 4.];

//     let c1 = na::center(&a, &b);
//     let c2 = na::center(&b, &c);

//     let cc1 = na::center(&c1, &c);
//     let cc2 = na::center(&c2, &a);

//     println!("{}", cc1);
//     println!("{}", cc2);

//     // assert!(na::center(&c1, &c) == na::center(&c2, &a), "SHIT")
// }
