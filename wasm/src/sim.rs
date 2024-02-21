use na::{Point2, Vector2};
use nalgebra as na;
use rand::{
    distributions::{Distribution, Uniform},
    Rng,
};
use serde::{ser::SerializeSeq, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;

#[derive(Debug)]
struct Particle {
    pos: Point2<f64>,
    vel: Vector2<f64>,
}

impl Particle {
    fn new(world_size: u32) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            pos: na::point![
                rng.gen_range(0..world_size) as f64,
                rng.gen_range(0..world_size) as f64,
            ],
            vel: na::vector![0., 0.],
        }
    }

    /// Get the force another particle exerts on this particle given the gravitational constant g.
    fn force(&self, other: &Particle, g: f64) -> Vector2<f64> {
        let d = (self.pos - other.pos).abs();
        let md = d.magnitude();
        if md > 0. && md < 80. {
            d * (g / md) * 0.5
        } else {
            na::vector![0., 0.]
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
}

impl Culture {
    fn new(color: String, world_size: u32, population: usize) -> Self {
        let particles = std::iter::repeat_with(|| Particle::new(world_size))
            .take(population)
            .collect::<Vec<_>>();

        Self { color, particles }
    }

    fn force(&self, other: &Culture, g: f64) -> Vec<Vector2<f64>> {
        self.particles
            .iter()
            .map(|p1| {
                // Accumulate force on p1
                other
                    .particles
                    .iter()
                    .fold(na::vector![0., 0.], |acc, p2| acc + p1.force(p2, g))
            })
            .collect()
    }
}

#[derive(Debug, Serialize)]
#[wasm_bindgen]
pub struct PetriDish {
    world_size: u32,
    population_per_culture: usize,
    cultures: Vec<Culture>,
    #[serde(skip)]
    gravity_mesh: Vec<Vec<f64>>,
}

#[wasm_bindgen]
impl PetriDish {
    #[wasm_bindgen(constructor)]
    pub fn new(colors: Vec<String>, world_size: u32, population: usize) -> Self {
        let cultures = colors
            .into_iter()
            .map(|color| Culture::new(color, world_size, population))
            .collect::<Vec<_>>();
        let num_cultures = cultures.len();
        let mut rng = rand::thread_rng();
        let distr = Uniform::new_inclusive(-1., 1.);
        let gravity_mesh = (0..num_cultures)
            .map(|_| distr.sample_iter(&mut rng).take(num_cultures).collect())
            .collect();
        Self {
            world_size,
            population_per_culture: population,
            cultures,
            gravity_mesh,
        }
    }

    // pub fn step(&mut self) {
    //     let arc_cultures = self
    //         .cultures
    //         .iter()
    //         .map(|c| Arc::new(RwLock::new(c)))
    //         .collect::<Vec<_>>();
    //     let arc_gravity = self
    //         .gravity_mesh
    //         .iter()
    //         .map(|g| Arc::new(RwLock::new(g)))
    //         .collect::<Vec<_>>();
    //     let mut handles = Vec::with_capacity(self.cultures.len());
    //     for i in 0..self.cultures.len() {
    //         let cultures = arc_cultures.iter().map(Arc::clone).collect::<Vec<_>>();
    //         let gravity_vecs = arc_gravity.iter().map(Arc::clone).collect::<Vec<_>>();
    //         let handle = thread::spawn(move || {
    //             let c1 = cultures[i].read().unwrap();
    //             let gravity_vec = gravity_vecs[i].read().unwrap();
    //             for j in 0..cultures.len() {
    //                 let c2 = cultures[j].read().unwrap();
    //                 c1.force(&c2, gravity_vec[j]);
    //             }
    //         });
    //         handles.push(handle);
    //     }
    //     for handle in handles {
    //         handle.join();
    //     }
    //     ()
    // }

    pub fn step(&mut self) {
        // Generate force tensor
        let force_tensor: Vec<Vec<Vector2<f64>>> = self
            .cultures
            .iter()
            .enumerate()
            .map(|(i, c1)| {
                let initial_forces = vec![na::vector![0., 0.]; self.population_per_culture];
                self.cultures
                    .iter()
                    .enumerate()
                    .fold(initial_forces, |acc, (j, c2)| {
                        let forces = c1.force(c2, self.gravity_mesh[i][j].clone());
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
                p.pos += p.vel;
                if p.pos.x <= 0. {
                    p.vel.x = (p.vel.x as f64).abs();
                    // p.pos.x = 0.;
                }
                if p.pos.x >= self.world_size as f64 {
                    p.vel.x = -(p.vel.x as f64).abs();
                    // p.pos.x = self.world_size as f64;
                }
                if p.pos.y <= 0. {
                    p.vel.y = (p.vel.y as f64).abs();
                    // p.pos.y = 0.;
                }
                if p.pos.y >= self.world_size as f64 {
                    p.vel.y = -(p.vel.y as f64).abs();
                    // p.pos.y = self.world_size as f64;
                }
            }
        }
    }

    pub fn cultures(&self) -> String {
        serde_json::to_string(&self.cultures).unwrap()
    }

    pub fn gravity_mesh(&self) -> String {
        serde_json::to_string(&self.gravity_mesh).unwrap()
    }
}

// #[test]
// fn test() {
//     let mut pd = PetriDish::new(
//         vec!["red".to_string(), "blue".to_string(), "green".to_string()],
//         200,
//         200,
//     );
//     // println!("Before: {:#}", serde_json::to_value(&pd).unwrap());
//     // let now = Instant::now();
//     for _ in 0..10 {
//         pd.step();
//         // println!("STEP {} {:.2?}", i+1, now.elapsed());
//     }
//     // println!("After: {:#}", serde_json::to_value(&pd).unwrap());
//     // println!("Gravity Mesh: {:#}", pd.gravity_mesh())
// }

#[test]
fn test_na() {
    let culture = Culture::new("a".to_string(), 100, 2);
    let is_same = std::ptr::eq(&culture.particles[0], &culture.particles[1]);
    println!("is same? {}", is_same);
    // let mut v = na::vector![0., 0.];
    // println!("{}", v);
    // v.x = 5.;
    // v.x = (v.x as f64).abs();
    // println!("{}", v);
    // let mut p = na::point![0., 0.];
    // println!("{}", p);
    // p.x = 5.;
    // println!("{}", p);
}
