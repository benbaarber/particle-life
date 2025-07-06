use macroquad::color::Color;
use random_color::RandomColor;

/// Generate a random hex color
pub fn random_color() -> Color {
    let [r, g, b, a] = RandomColor::new().into_rgba_array();
    Color::from_rgba(r, g, b, a)
}
