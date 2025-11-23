use rand_distr::{Distribution, Uniform};
use random_color::RandomColor;

pub fn random_color() -> [f32; 4] {
    RandomColor::new().into_f32_rgba_array()
}

pub fn random_gravity_mesh_flat(num_cultures: usize) -> Vec<f32> {
    let mut rng = rand::rng();
    let distr = Uniform::new_inclusive(-1., 1.).unwrap();
    distr
        .sample_iter(&mut rng)
        .take(num_cultures * num_cultures)
        .collect()
}
