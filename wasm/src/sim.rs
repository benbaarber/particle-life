#![allow(unused)]
use rand::Rng;

pub struct Vector2D {
    pub x: f32,
    pub y: f32,
}

impl Vector2D {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

pub struct Particle {
    pub x: u32,
    pub y: u32,
    pub vx: u32,
    pub vy: u32,
}

impl Particle {
    pub fn new(world_size: u32) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            x: rng.gen_range(0..world_size),
            y: rng.gen_range(0..world_size),
            vx: 0,
            vy: 0,
        }
    }

    /// Get the force another particle exerts on this particle given the gravitational constant g.
    pub fn force(&self, other: &Particle, g: f32) -> Vector2D {
        let dx = (self.x - other.x) as f32;
        let dy = (self.y - other.y) as f32;
        let d = dx.hypot(dy);
        if d > 0. && d < 80. {
            let fx = g * (1./dx);
            let fy = g * (1./dy);
            Vector2D::new(fx, fy)
        } else {
            Vector2D::new(0., 0.)
        }
    }
}

pub struct Culture {
    pub color: String,
    pub particles: Vec<Particle>,
}

impl Culture {
    pub fn new(color: &str, world_size: u32, population: usize) -> Self {
        let particles = std::iter::repeat_with(|| Particle::new(world_size))
            .take(population)
            .collect::<Vec<_>>();

        Self {
            color: color.to_owned(),
            particles,
        }
    }

    
}