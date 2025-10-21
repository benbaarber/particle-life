use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuParams {
    pub num_cultures: u32,
    pub culture_size: u32,
    pub theta2: f32,
    pub aoe2: f32,
}

pub struct GpuCompute {
    num_particles: u64,
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::ComputePipeline,
    bind_group: wgpu::BindGroup,
    particle_buffer: wgpu::Buffer,
    force_buffer: wgpu::Buffer,
    download_buffer: wgpu::Buffer,
}

impl GpuCompute {
    pub async fn new(params: GpuParams, gravity_mesh: &[f32]) -> Self {
        let instance = wgpu::Instance::new(&Default::default());
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: None,
            })
            .await
            .unwrap();
        let (device, queue) = adapter.request_device(&Default::default()).await.unwrap();

        let num_particles = (params.num_cultures * params.culture_size) as u64;
        let particle_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: num_particles * size_of::<[f32; 2]>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let force_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: num_particles * size_of::<[f32; 2]>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::bytes_of(&params),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let gravity_mesh_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(gravity_mesh),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let download_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: force_buffer.size(),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let shader = device.create_shader_module(wgpu::include_wgsl!("force.wgsl"));
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Force Computation"),
            layout: None,
            module: &shader,
            entry_point: None,
            compilation_options: Default::default(),
            cache: None,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: particle_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: force_buffer.as_entire_binding(),
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

        Self {
            num_particles,
            device,
            queue,
            pipeline,
            bind_group,
            particle_buffer,
            force_buffer,
            download_buffer,
        }
    }

    pub fn run(&self, particles: &[[f32; 2]]) -> Vec<[f32; 2]> {
        assert_eq!(particles.len() as u64, self.num_particles);

        self.queue
            .write_buffer(&self.particle_buffer, 0, bytemuck::cast_slice(particles));

        let mut encoder = self.device.create_command_encoder(&Default::default());

        let workgroup_count = particles.len().div_ceil(64);
        let mut compute_pass = encoder.begin_compute_pass(&Default::default());
        compute_pass.set_pipeline(&self.pipeline);
        compute_pass.set_bind_group(0, &self.bind_group, &[]);
        compute_pass.dispatch_workgroups(workgroup_count as u32, 1, 1);
        drop(compute_pass);

        encoder.copy_buffer_to_buffer(
            &self.force_buffer,
            0,
            &self.download_buffer,
            0,
            self.force_buffer.size(),
        );
        encoder.map_buffer_on_submit(&self.download_buffer, wgpu::MapMode::Read, .., |_| {});

        let command_buffer = encoder.finish();
        self.queue.submit([command_buffer]);

        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .unwrap();

        let result = {
            let data = self.download_buffer.get_mapped_range(..);
            let result: &[[f32; 2]] = bytemuck::cast_slice(&data);
            result.to_vec()
        };

        self.download_buffer.unmap();
        result
    }
}
