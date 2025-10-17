use macroquad::color::Color;
use random_color::RandomColor;

/// Generate a random hex color
pub fn random_color() -> Color {
    let [r, g, b, a] = RandomColor::new().into_rgba_array();
    Color::from_rgba(r, g, b, a)
}

// #[cfg(not(target_arch = "wasm32"))]
// pub fn is_left_mouse_down() -> bool {
//     macroquad::input::is_mouse_button_down(macroquad::input::MouseButton::Left)
// }
//
// #[cfg(not(target_arch = "wasm32"))]
// pub fn is_right_mouse_down() -> bool {
//     macroquad::input::is_mouse_button_down(macroquad::input::MouseButton::Right)
// }
//
// #[cfg(not(target_arch = "wasm32"))]
// pub fn mouse_position() -> Vec2 {
//     let (mx, my) = macroquad::input::mouse_position();
//     vec2(mx, my)
// }
