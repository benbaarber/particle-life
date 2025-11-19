use particle_life::wg::app::{App, GpuParams};
use winit::event_loop::{ControlFlow, EventLoop};

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let num_cultures = 8;
    let culture_size = 5000;
    let aoe = 30.0; // area of effect of forces
    let damping = 0.25; // velocity damping (0.0-1.0, smaller = slower)
    let params = GpuParams::new(num_cultures, culture_size, aoe, damping);
    let mut app = App::new(params);
    event_loop.run_app(&mut app).unwrap();
}
