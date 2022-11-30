mod camera;
mod voxel;
mod instance;
mod function;

use instance::InstanceRaw;
use voxel::{INDICES};
use wgpu::{Device, Surface, SurfaceConfiguration, Queue, SurfaceError, Instance, Backends, RenderPipeline, Buffer, util::{DeviceExt, BufferInitDescriptor}, BindGroup};
use winit::{event::{*,}, event_loop::{ControlFlow, EventLoop}, window::WindowBuilder,window::{Window}, dpi::PhysicalSize, event::{WindowEvent}};

use crate::voxel::{LENGTH, RES};

//STATE STUFF
#[allow(dead_code)]
struct State {
    surface: Surface,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    size: PhysicalSize<u32>,
    render_pipeline: RenderPipeline,

    vertex_buffer: Buffer,
    index_buffer: Buffer,
    num_indices: u32,

    camera: camera::Camera,
    camera_controller: camera::CameraController,
    camera_uniform: camera::CameraUniform,
    camera_buffer: Buffer,
    camera_bind_group: BindGroup,

    vertices: Vec<voxel::Vertex>,
    instances: Vec<instance::Instance>,
    faces: Vec<(i16, i16, i16)>,
    new_instances: Vec<instance::Instance>,
    function_index: i16,
    instance_data: Vec<InstanceRaw>,
    instance_buffer: Buffer,
    frame_number: u32,
    a: f32,
}

impl State {

    async fn new(window: &Window) -> Self {

    //Input variables - this values work well enough for my taste
        let speed = 0.0007;
        let sensitivity = 0.006;
        let scroll_sensitivity = 0.009;

    //WINDOW
        let size = window.inner_size();

        let instance = Instance::new(Backends::all());

        let surface = unsafe {instance.create_surface(window)};

        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor{
                features: wgpu::Features::MAPPABLE_PRIMARY_BUFFERS,
                limits: if cfg!(targer_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                label: None
            },
            None,)
            .await.unwrap();

        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter) [0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        };
        surface.configure(&device, &config);


    //CAMERA
        let camera = camera::Camera::new((0.0, 0.0, 20.0), (0.0, 0.0, 0.0), (config.width / config.height) as f32,cgmath::Deg(45.0), 0.1, 100.0);

        let camera_controller = camera::CameraController::new(speed, sensitivity, scroll_sensitivity);

        let mut camera_uniform = camera::CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Camera Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
                }
            ],
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }
            ],
        });

    //RENDERING
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("orbsh.wgsl").into()),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),

            layout: Some(&render_pipeline_layout),

            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    voxel::Vertex::desc(), InstanceRaw::desc()
                ],
            },

            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },

            depth_stencil: None,

            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },

            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),

            multiview: None,
        });

        let vertices = voxel::generate_voxel();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&INDICES),
                usage: wgpu::BufferUsages::INDEX,
        });

    //INSTANCING
        let a = 0.0;

        let num_indices = INDICES.len() as u32;

        let instances: Vec<instance::Instance> = vec![];

        let faces: Vec<(i16, i16, i16)> = vec![];

        let new_instances: Vec<instance::Instance> = vec![];

        let function_index: i16 = -LENGTH * RES as i16;

        let instance_data = instances.iter().map(instance::Instance::to_raw).collect::<Vec<_>>();

        let instance_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instance_data) ,
            usage: wgpu::BufferUsages::VERTEX,
        });

        let frame_number = 0;

        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            camera_controller,
            vertices,
            instances,
            faces,
            new_instances,
            function_index,
            instance_data,
            instance_buffer,
            frame_number,
            a,
        }
    }

    //RESIZE WINDOW FUNCTION
    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {

        if new_size.width > 0 && new_size.height > 0 {
            self.camera.resize(new_size.width, new_size.height);
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    //INPUTS
    fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::MouseWheel { delta, .. } => {
                self.camera_controller.process_scroll(delta);
                true
            }

            WindowEvent::MouseInput {button, state, ..} => {
                self.camera_controller.process_buttons(*button, *state)
            }
            _ => false,
        }
    }

    //UPDATE THING
    fn update(&mut self, dt: std::time::Duration) {

        self.frame_number += 1;

        self.camera_controller.update_camera(&mut self.camera, dt);
        self.camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));

        if self.frame_number == 1 {println!("Create");}

        if self.frame_number <= 4 {

            let mut function_result = function::orbital(self.function_index, &self.faces, self.a);

            self.new_instances.append(&mut function_result.0);
            self.function_index = function_result.1;
            self.faces = function_result.2;

            //println!("{:?}", self.new_instances.len());
            //println!("Faces: {:?}", self.faces.len());
            //println!("Index: {:?}", self.function_index);

        }

        

        if self.frame_number == 5 {

            println!("Data");

            self.instance_data = self.new_instances.iter().map(instance::Instance::to_raw).collect::<Vec<_>>();
            
        }
        if self.frame_number == 6 {
            
            println!("Commit");

            self.instances = vec![];

            self.faces = vec![];

            self.instances.append(&mut self.new_instances);

            self.instance_buffer = self.device.create_buffer_init(&BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&self.instance_data) ,
                usage: wgpu::BufferUsages::VERTEX,
            });
            self.a += 0.1; 
        }

        if self.frame_number == 6 {

            self.function_index = -LENGTH * RES as i16;
            self.frame_number = 0;
        }
        
    }

    //RENDER FUNCTION
    fn render(&mut self) -> Result<(), SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.01,
                            g: 0.01,
                            b: 0.01,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            render_pass.draw_indexed(0..self.num_indices, 0, 0..self.instances.len() as _);
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}


//RUN FUNCTION
pub async fn run() {
 env_logger::init();

    let event_loop = EventLoop::new();
    let title = "Orbital Simulation";
    let window = WindowBuilder::new()
        .with_title(title)
        .build(&event_loop)
        .unwrap();

    let mut state = State::new(&window).await;
    let mut last_render_time = instant::Instant::now();

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::MainEventsCleared => window.request_redraw(),
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion {delta}, ..
            } => {
                state.camera_controller.process_mouse(delta.0, delta.1)
            }
            Event::WindowEvent {
                ref event, window_id,
            } if window_id == window.id() && !state.input(event) => {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input: KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                    } => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged {new_inner_size, .. } => {
                        state.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }

            Event::RedrawRequested(window_id) if window_id == window.id() => {
                let now = instant::Instant::now();
                let dt = now - last_render_time;
                println!("{:?}", dt);
                last_render_time = now;
                match state.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => state.resize(state.size),
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                }
                state.update(dt);
                log::log!(log::Level::Info,"{:?}", state.camera.position);
            }
            _ => {}
        }
    });
}
