use macroquad::color::Color;
use rand_distr::{Distribution, Uniform};
use random_color::RandomColor;

/// Generate a random hex color
pub fn random_color() -> Color {
    let [r, g, b, a] = RandomColor::new().into_rgba_array();
    Color::from_rgba(r, g, b, a)
}

pub fn random_gravity_mesh(num_cultures: usize) -> Vec<Vec<f32>> {
    let mut rng = rand::rng();
    let distr = Uniform::new_inclusive(-1., 1.).unwrap();
    (0..num_cultures)
        .map(|_| distr.sample_iter(&mut rng).take(num_cultures).collect())
        .collect::<Vec<_>>()
}

pub fn random_gravity_mesh_flat(num_cultures: usize) -> Vec<f32> {
    let mut rng = rand::rng();
    let distr = Uniform::new_inclusive(-1., 1.).unwrap();
    distr
        .sample_iter(&mut rng)
        .take(num_cultures * num_cultures)
        .collect()
}
