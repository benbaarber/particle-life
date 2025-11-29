use std::{sync::Arc, time::Instant};

use anyhow::Result;
use rand::Rng;
use wgpu::util::DeviceExt;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

use crate::util::random_color;

const PHYS_DT: f32 = 1.0 / 60.0;
const MAX_ACC: f32 = 5.0 / 60.0;

pub fn run(params: GpuParams, mesh: Vec<f32>) {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::new(params, mesh);
    event_loop.run_app(&mut app).unwrap();
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuParams {
    pub bound: [f32; 2],
    pub num_cultures: u32,
    pub culture_size: u32,
    pub num_particles: u32,
    pub aoe: f32,
    pub aoe2: f32,
    pub damping: f32,
    pub bin_size: f32,
    pub grid_w: u32,
}

impl GpuParams {
    pub fn new(num_cultures: u32, culture_size: u32, aoe: f32, damping: f32) -> Self {
        let bound = [1000.0, 1000.0];
        let grid_w = f32::ceil(bound[0] / (aoe * 2.0));
        let bin_size = bound[0] / grid_w;
        Self {
            bound,
            num_cultures,
            culture_size,
            num_particles: num_cultures * culture_size,
            aoe,
            aoe2: aoe * aoe,
            damping,
            bin_size,
            grid_w: grid_w as u32,
        }
    }
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

struct ComputeState {
    bin_counts_buffer: wgpu::Buffer,
    particle_buffer_1: wgpu::Buffer,
    particle_buffer_2: wgpu::Buffer,
    count_pipeline: wgpu::ComputePipeline,
    offsets_pipeline: wgpu::ComputePipeline,
    build_pipeline: wgpu::ComputePipeline,
    force_pipeline: wgpu::ComputePipeline,
    general_bind: wgpu::BindGroup,
    particle_bind_1: wgpu::BindGroup,
    particle_bind_2: wgpu::BindGroup,
    particle_bind_swap: bool,
}

struct RenderState {
    pipeline: wgpu::RenderPipeline,
    bind: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    surface: wgpu::Surface<'static>,
    surface_format: wgpu::TextureFormat,
    window: Arc<Window>,
    size: PhysicalSize<u32>,
}

struct State {
    device: wgpu::Device,
    queue: wgpu::Queue,
    params: GpuParams,
    compute_state: ComputeState,
    render_state: RenderState,
    time_acc: f32,
    last_frame_t: Instant,
    phys_steps: u32,
    rend_steps: u32,
    last_sec: Instant,
    t: u32,
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

        let colors = (0..params.num_cultures)
            .map(|_| random_color())
            .collect::<Vec<_>>();
        let particles = (0..params.num_particles)
            .map(|_| GpuParticle::new(params.bound))
            .collect::<Vec<_>>();
        let num_bins = (params.grid_w * params.grid_w) as usize;

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
        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Params"),
            contents: bytemuck::bytes_of(&params),
            usage: U::UNIFORM,
        });
        let gravity_mesh_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Gravity Mesh"),
            contents: bytemuck::cast_slice(gravity_mesh),
            usage: U::STORAGE,
        });
        let bin_counts_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Bin Counts"),
            contents: &bytemuck::cast_slice(&vec![0f32; num_bins]),
            usage: U::STORAGE | U::COPY_DST,
        });
        let bin_ixs_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Bin Indices"),
            contents: &bytemuck::cast_slice(&vec![0f32; params.num_particles as usize]),
            usage: U::STORAGE,
        });
        let bin_offsets_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Bin Offsets"),
            contents: &bytemuck::cast_slice(&vec![0f32; num_bins + 1]),
            usage: U::STORAGE,
        });
        let bin_current_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Bin Current"),
            contents: &bytemuck::cast_slice(&vec![0f32; num_bins]),
            usage: U::STORAGE,
        });
        let bins_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Bins"),
            contents: &bytemuck::cast_slice(&vec![0f32; params.num_particles as usize]),
            usage: U::STORAGE,
        });
        let colors_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Colors"),
            contents: bytemuck::cast_slice(&colors),
            usage: U::STORAGE,
        });
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertices"),
            contents: bytemuck::cast_slice(&particles),
            usage: U::VERTEX | U::COPY_DST,
        });

        let group0_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Compute Group 0 Layout"),
            entries: &[
                // params
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // gravity mesh
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // bin counts
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // bin ixs
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // bin offsets
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // bin current
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // bins
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let group1_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Group 1 Layout"),
            entries: &[
                // particles
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // particles out
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Compute Pipeline Layout"),
            bind_group_layouts: &[&group0_layout, &group1_layout],
            push_constant_ranges: &[],
        });

        let cshader = device.create_shader_module(wgpu::include_wgsl!("shaders/compute.wgsl"));

        let count_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Count Bins"),
            layout: Some(&pipeline_layout),
            module: &cshader,
            entry_point: Some("compute_bin_ixs_and_counts"),
            compilation_options: Default::default(),
            cache: None,
        });

        let offsets_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Offsets"),
            layout: Some(&pipeline_layout),
            module: &cshader,
            entry_point: Some("compute_bin_offsets"),
            compilation_options: Default::default(),
            cache: None,
        });

        let build_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Build Bins"),
            layout: Some(&pipeline_layout),
            module: &cshader,
            entry_point: Some("build_bin"),
            compilation_options: Default::default(),
            cache: None,
        });

        let force_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Forces"),
            layout: Some(&pipeline_layout),
            module: &cshader,
            entry_point: Some("compute_force"),
            compilation_options: Default::default(),
            cache: None,
        });

        let compute_general_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Compute General Bind Group"),
            layout: &group0_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: gravity_mesh_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: bin_counts_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: bin_ixs_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: bin_offsets_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: bin_current_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: bins_buffer.as_entire_binding(),
                },
            ],
        });

        let compute_particle_bind_1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Compute Particle Bind Group 1"),
            layout: &group1_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: particle_buffer_1.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: particle_buffer_2.as_entire_binding(),
                },
            ],
        });

        let compute_particle_bind_2 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Compute Particle Bind Group 2"),
            layout: &force_pipeline.get_bind_group_layout(1),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: particle_buffer_2.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: particle_buffer_1.as_entire_binding(),
                },
            ],
        });

        let compute_state = ComputeState {
            bin_counts_buffer,
            particle_buffer_1,
            particle_buffer_2,
            count_pipeline,
            offsets_pipeline,
            build_pipeline,
            force_pipeline,
            general_bind: compute_general_bind,
            particle_bind_1: compute_particle_bind_1,
            particle_bind_2: compute_particle_bind_2,
            particle_bind_swap: false,
        };

        let surface = instance.create_surface(Arc::clone(&window))?;
        let cap = surface.get_capabilities(&adapter);
        let surface_format = cap.formats[0];

        let rshader = device.create_shader_module(wgpu::include_wgsl!("shaders/render.wgsl"));

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

        let render_state = RenderState {
            pipeline: render_pipeline,
            bind: render_bind_group,
            vertex_buffer,
            surface,
            surface_format,
            window,
            size,
        };

        let gc = Self {
            device,
            queue,
            params,
            compute_state,
            render_state,
            time_acc: 0.0,
            last_frame_t: Instant::now(),
            phys_steps: 0,
            rend_steps: 0,
            last_sec: Instant::now(),
            t: 0,
        };

        gc.configure_surface();

        Ok(gc)
    }

    fn get_window(&self) -> &Window {
        &self.render_state.window
    }

    fn configure_surface(&self) {
        let rs = &self.render_state;
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: rs.surface_format,
            view_formats: vec![rs.surface_format.add_srgb_suffix()],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            width: rs.size.width,
            height: rs.size.height,
            desired_maximum_frame_latency: 3,
            present_mode: wgpu::PresentMode::AutoVsync,
        };
        rs.surface.configure(&self.device, &surface_config);
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.render_state.size = new_size;

        // reconfigure the surface
        self.configure_surface();
    }

    pub fn compute(&mut self) -> wgpu::CommandBuffer {
        let c = &self.compute_state;
        let (bind_group, particle_out_buffer) = if c.particle_bind_swap {
            (&c.particle_bind_2, &c.particle_buffer_1)
        } else {
            (&c.particle_bind_1, &c.particle_buffer_2)
        };

        let mut encoder = self.device.create_command_encoder(&Default::default());

        encoder.clear_buffer(&c.bin_counts_buffer, 0, None);

        let workgroup_count = self.params.num_particles.div_ceil(64);

        let mut cpass = encoder.begin_compute_pass(&Default::default());
        cpass.set_bind_group(0, &c.general_bind, &[]);
        cpass.set_bind_group(1, bind_group, &[]);

        cpass.set_pipeline(&c.count_pipeline);
        cpass.dispatch_workgroups(workgroup_count, 1, 1);

        cpass.set_pipeline(&c.offsets_pipeline);
        cpass.dispatch_workgroups(1, 1, 1);

        cpass.set_pipeline(&c.build_pipeline);
        cpass.dispatch_workgroups(workgroup_count, 1, 1);

        cpass.set_pipeline(&c.force_pipeline);
        cpass.dispatch_workgroups(workgroup_count, 1, 1);

        drop(cpass);

        encoder.copy_buffer_to_buffer(
            particle_out_buffer,
            0,
            &self.render_state.vertex_buffer,
            0,
            particle_out_buffer.size(),
        );

        self.compute_state.particle_bind_swap = !c.particle_bind_swap;

        encoder.finish()
    }

    pub fn render(&mut self) {
        let r = &self.render_state;
        // Create texture view
        let surface_texture = r
            .surface
            .get_current_texture()
            .expect("failed to acquire next swapchain texture");
        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor {
                format: Some(r.surface_format.add_srgb_suffix()),
                ..Default::default()
            });

        let mut encoder = self.device.create_command_encoder(&Default::default());
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

        rpass.set_pipeline(&r.pipeline);
        rpass.set_bind_group(0, &r.bind, &[]);
        rpass.set_vertex_buffer(0, r.vertex_buffer.slice(..));
        rpass.draw(0..6, 0..self.params.num_particles);

        drop(rpass);

        self.queue.submit([encoder.finish()]);
        r.window.pre_present_notify();
        surface_texture.present();
    }

    pub fn step(&mut self) {
        let now = Instant::now();
        let dur = now.duration_since(self.last_frame_t).as_secs_f32();
        self.last_frame_t = now;

        if now.duration_since(self.last_sec).as_secs_f32() >= 1.0 {
            self.t += 1;
            println!(
                "t={}\nPhysics FPS: {}\nRender FPS: {}",
                self.t, self.phys_steps, self.rend_steps
            );
            self.phys_steps = 0;
            self.rend_steps = 0;
            self.last_sec = now;
        }

        self.time_acc += dur;
        self.time_acc = f32::min(self.time_acc, MAX_ACC);

        let mut cmd_bufs = vec![];
        while self.time_acc >= PHYS_DT {
            let cmd = self.compute();
            cmd_bufs.push(cmd);
            self.phys_steps += 1;
            self.time_acc -= PHYS_DT;
        }

        if cmd_bufs.len() > 0 {
            self.queue.submit(cmd_bufs);
        }

        self.render();
        self.rend_steps += 1;
    }
}

pub struct App {
    params: GpuParams,
    state: Option<State>,
    mesh: Vec<f32>,
}

impl App {
    pub fn new(params: GpuParams, mesh: Vec<f32>) -> Self {
        Self {
            params,
            state: None,
            mesh,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );

        let state = pollster::block_on(State::new(Arc::clone(&window), self.params, &self.mesh));
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
                state.step();
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
