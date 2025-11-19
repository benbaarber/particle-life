use std::sync::Arc;

use anyhow::Result;
use rand::Rng;
use wgpu::util::DeviceExt;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{KeyEvent, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

use crate::util::{random_color, random_gravity_mesh_flat};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuParams {
    pub bound: [f32; 2],
    pub num_cultures: u32,
    pub culture_size: u32,
    pub theta2: f32,
    pub aoe2: f32,
    pub damping: f32,
    pub _padding: u32,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuParticle {
    pos: [f32; 2],
    vel: [f32; 2],
}

impl GpuParticle {
    pub fn new(bound: [f32; 2]) -> Self {
        let mut rng = rand::rng();
        Self {
            pos: [
                rng.random_range(0.0..bound[0]),
                rng.random_range(0.0..bound[1]),
            ],
            vel: [rng.random_range(-1.0..1.0), rng.random_range(-1.0..1.0)],
            // vel: [1.0, 1.0],
        }
    }
    pub fn vertex_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            // can also use wgpu::vertex_attr_array![] macro
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x2,
            }],
        }
    }
}

pub struct State {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    surface_format: wgpu::TextureFormat,
    window: Arc<Window>,
    size: PhysicalSize<u32>,
    particle_buffer_1: wgpu::Buffer,
    particle_buffer_2: wgpu::Buffer,
    vertex_buffer: wgpu::Buffer,
    compute_pipeline: wgpu::ComputePipeline,
    compute_bind_group_1: wgpu::BindGroup,
    compute_bind_group_2: wgpu::BindGroup,
    compute_bind_group_swap: bool,
    render_pipeline: wgpu::RenderPipeline,
    render_bind_group: wgpu::BindGroup,
    num_particles: u32,
}

impl State {
    pub async fn new(window: Arc<Window>, params: GpuParams, gravity_mesh: &[f32]) -> Result<Self> {
        let instance = wgpu::Instance::new(&Default::default());
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: None,
            })
            .await?;
        let (device, queue) = adapter.request_device(&Default::default()).await?;

        let num_particles = (params.num_cultures * params.culture_size) as u64;
        let colors = (0..params.num_cultures)
            .map(|_| random_color().into())
            .collect::<Vec<[f32; 4]>>();
        let particles = (0..num_particles)
            .map(|_| GpuParticle::new(params.bound))
            .collect::<Vec<_>>();

        use wgpu::BufferUsages as U;
        let particle_buffer_1 = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Particles"),
            contents: bytemuck::cast_slice(&particles),
            usage: U::STORAGE | U::COPY_SRC,
        });
        let particle_buffer_2 = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Particles"),
            contents: bytemuck::cast_slice(&particles),
            usage: U::STORAGE | U::COPY_SRC,
        });
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertices"),
            contents: bytemuck::cast_slice(&particles),
            usage: U::VERTEX | U::COPY_DST,
        });
        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Params"),
            contents: bytemuck::bytes_of(&params),
            usage: U::UNIFORM,
        });
        let colors_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Colors"),
            contents: bytemuck::cast_slice(&colors),
            usage: U::STORAGE,
        });
        let gravity_mesh_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Gravity Mesh"),
            contents: bytemuck::cast_slice(gravity_mesh),
            usage: U::STORAGE,
        });

        let cshader = device.create_shader_module(wgpu::include_wgsl!("compute.wgsl"));
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Force Computation"),
            layout: None,
            module: &cshader,
            entry_point: None,
            compilation_options: Default::default(),
            cache: None,
        });

        let compute_bind_group_1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &compute_pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: particle_buffer_1.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: particle_buffer_2.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: gravity_mesh_buffer.as_entire_binding(),
                },
            ],
        });

        let compute_bind_group_2 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &compute_pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: particle_buffer_2.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: particle_buffer_1.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: gravity_mesh_buffer.as_entire_binding(),
                },
            ],
        });
        let surface = instance.create_surface(Arc::clone(&window))?;
        let cap = surface.get_capabilities(&adapter);
        let surface_format = cap.formats[0];

        let rshader = device.create_shader_module(wgpu::include_wgsl!("render.wgsl"));

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: None,
            vertex: wgpu::VertexState {
                module: &rshader,
                entry_point: Some("vs_main"),
                buffers: &[GpuParticle::vertex_layout()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &rshader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        let render_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &render_pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: colors_buffer.as_entire_binding(),
                },
            ],
        });

        let size = window.inner_size();

        let gc = Self {
            device,
            queue,
            surface,
            surface_format,
            window,
            size,
            particle_buffer_1,
            particle_buffer_2,
            vertex_buffer,
            compute_pipeline,
            compute_bind_group_1,
            compute_bind_group_2,
            compute_bind_group_swap: false,
            render_pipeline,
            render_bind_group,
            num_particles: params.num_cultures * params.culture_size,
        };

        gc.configure_surface();

        Ok(gc)
    }

    fn get_window(&self) -> &Window {
        &self.window
    }

    fn configure_surface(&self) {
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.surface_format,
            // Request compatibility with the sRGB-format texture view we‘re going to create later.
            view_formats: vec![self.surface_format.add_srgb_suffix()],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            width: self.size.width,
            height: self.size.height,
            desired_maximum_frame_latency: 2,
            present_mode: wgpu::PresentMode::AutoVsync,
        };
        self.surface.configure(&self.device, &surface_config);
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;

        // reconfigure the surface
        self.configure_surface();
    }

    pub fn compute(&mut self) {
        let mut encoder = self.device.create_command_encoder(&Default::default());

        let workgroup_count = self.num_particles.div_ceil(64);
        let mut cpass = encoder.begin_compute_pass(&Default::default());
        cpass.set_pipeline(&self.compute_pipeline);
        if self.compute_bind_group_swap {
            cpass.set_bind_group(0, &self.compute_bind_group_2, &[]);
        } else {
            cpass.set_bind_group(0, &self.compute_bind_group_1, &[]);
        }
        cpass.dispatch_workgroups(workgroup_count as u32, 1, 1);
        drop(cpass);

        encoder.copy_buffer_to_buffer(
            &self.particle_buffer_2,
            0,
            &self.vertex_buffer,
            0,
            self.particle_buffer_2.size(),
        );

        let command_buffer = encoder.finish();
        self.queue.submit([command_buffer]);

        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .unwrap();

        std::mem::swap(&mut self.particle_buffer_1, &mut self.particle_buffer_2);
        self.compute_bind_group_swap = !self.compute_bind_group_swap;
    }

    pub fn render(&mut self) {
        // Create texture view
        let surface_texture = self
            .surface
            .get_current_texture()
            .expect("failed to acquire next swapchain texture");
        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor {
                // Without add_srgb_suffix() the image we will be working with
                // might not be "gamma correct".
                format: Some(self.surface_format.add_srgb_suffix()),
                ..Default::default()
            });

        // Renders a GREEN screen
        let mut encoder = self.device.create_command_encoder(&Default::default());
        // Create the renderpass which will clear the screen.
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &texture_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // If you wanted to call any drawing commands, they would go here.
        rpass.set_pipeline(&self.render_pipeline);
        rpass.set_bind_group(0, &self.render_bind_group, &[]);
        rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        rpass.draw(0..6, 0..self.num_particles);

        // End the renderpass.
        drop(rpass);

        // Submit the command in the queue to execute
        self.queue.submit([encoder.finish()]);
        self.window.pre_present_notify();
        surface_texture.present();
    }

    pub fn step(&mut self) -> Result<()> {
        self.compute();
        self.render();
        Ok(())
    }
}
#[derive(Default)]
pub struct App {
    state: Option<State>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Create window object
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );

        let aoe = 50.0;
        let params = GpuParams {
            num_cultures: 8,
            culture_size: 5000,
            theta2: 0.9 * 0.9,
            aoe2: aoe * aoe,
            damping: 0.3,
            bound: [1000.0, 1000.0], // this value is hardcoded into the render shader
            _padding: 0,
        };
        let mesh = random_gravity_mesh_flat(params.num_cultures as usize);
        println!("Gravity mesh: {:?}", mesh);
        let state = pollster::block_on(State::new(Arc::clone(&window), params, &mesh));
        self.state = Some(state.unwrap());

        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let state = self.state.as_mut().unwrap();
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                state.step().unwrap();
                // Emits a new redraw requested event.
                state.get_window().request_redraw();
            }
            WindowEvent::Resized(size) => {
                // Reconfigures the size of the surface. We do not re-render
                // here as this event is always followed up by redraw request.
                state.resize(size);
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => handle_key(event_loop, code, key_state.is_pressed()),
            _ => (),
        }
    }
}

fn handle_key(event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
    if !is_pressed {
        return;
    }
    match code {
        KeyCode::KeyQ => event_loop.exit(),
        _ => (),
    }
}
