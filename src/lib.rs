mod camera;
mod voxel;
mod instance;
mod function;
mod orbitals;
mod interface;

use egui::{FontDefinitions, epaint::ImageDelta, TextureId};
use egui_wgpu::{wgpu::{self}, renderer::ScreenDescriptor};
use egui_winit_platform::Platform;
use instance::InstanceRaw;
use interface::Guindow;
use orbitals::Orbital;
use voxel::INDICES;
use egui_wgpu::wgpu::{Surface, SurfaceConfiguration, Queue, SurfaceError, Backends, RenderPipeline, Buffer, util::{DeviceExt, BufferInitDescriptor}, BindGroup};
use winit::{event::*, event_loop::{ControlFlow, EventLoop}, window::WindowBuilder,window::{Window}, dpi::{PhysicalSize, PhysicalPosition}, event::{WindowEvent}, monitor::MonitorHandle};

use crate::{voxel::{LENGTH}, interface::Gui};

//STATE STUFF
#[allow(dead_code)]
struct State {
    surface: Surface,
    device: egui_wgpu::wgpu::Device,
    queue: Queue,
    config: SurfaceConfiguration,
    size: PhysicalSize<u32>,
    platform: Platform,

    render_pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    num_indices: u32,

    camera: camera::Camera,
    camera_controller: camera::CameraController,
    camera_uniform: camera::CameraUniform,
    camera_buffer: Buffer,
    camera_bind_group: BindGroup,
    instance_camera: camera::Camera,

    vertices: Vec<voxel::Vertex>,
    instances: Vec<instance::Instance>,
    new_instances: Vec<instance::Instance>,
    function_index: i16,
    instance_data: Vec<InstanceRaw>,
    instance_buffer: Buffer,
    current_resolution: f32,
    current_bohr: f32,
    last_dt: (Vec<f32>, usize),

    orbital_array: Vec<Orbital>,
}

impl State {

    async fn new(window: &Window) -> Self {

    //Input variables - this values work well enough for my taste
        let speed = 0.055;
        let sensitivity = 0.006;
        let scroll_sensitivity = 0.009;

    //WINDOW
        let size = window.inner_size();

        let instance = egui_wgpu::wgpu::Instance::new(Backends::all());

        let surface = unsafe {instance.create_surface(window)};

        let adapter = pollster::block_on(instance.request_adapter(
            &egui_wgpu::wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        )).unwrap();

        let (device, queue) = adapter.request_device(
            &egui_wgpu::wgpu::DeviceDescriptor{
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
        };
        surface.configure(&device, &config);

        let platform = Platform::new(egui_winit_platform::PlatformDescriptor {
            physical_width: size.width as u32,
            physical_height: size.height as u32,
            scale_factor: window.scale_factor(),
            font_definitions: FontDefinitions::default(),
            style: Default::default(),
        });

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

        let instance_camera = camera;

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
                alpha_to_coverage_enabled: true,
            },

            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::all(),
                })],
            }),

            multiview: None,
        });

        let vertices = voxel::generate_face(5.0);

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
        let num_indices = INDICES.len() as u32;

        let instances: Vec<instance::Instance> = vec![];

        let new_instances: Vec<instance::Instance> = vec![];

        let function_index: i16 = 0;

        let instance_data = instances.iter().map(instance::Instance::to_raw).collect::<Vec<_>>();

        let instance_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instance_data) ,
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            surface,
            device,
            queue,
            config,
            size,
            platform,

            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,

            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            camera_controller,
            instance_camera,

            vertices,
            instances,
            new_instances,
            function_index,
            instance_data,
            instance_buffer,
            current_resolution: 5.0,
            current_bohr: 0.25,
            last_dt: (vec![0.016; 6], 0),

            orbital_array: vec![],
        }
    }

//RESIZE WINDOW FUNCTION - It resizes the window
    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {

        if new_size.width > 0 && new_size.height > 0 {
            self.camera.resize(new_size.width, new_size.height);
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

//INPUTS - Basically checks if any meaningful inputs are pressed, if so it runs some code
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

//UPDATE FUNCTION - Called every frame, updates processes
    fn update(&mut self, gui_app: &Guindow, dt: f32) {

    //
        self.last_dt.0[self.last_dt.1] = dt;
        self.last_dt.1 = (self.last_dt.1 + 1) % 6;
        let average_dt: f32 = self.last_dt.0.clone().into_iter().sum::<f32>() / 6.0;

    //Update to the camera controller & view
        self.camera_controller.update_camera(&mut self.camera, average_dt);
        self.camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));
        self.orbital_array = gui_app.orbitals.clone();
    
    //Update to all the render logic
            if self.function_index == (-LENGTH) * self.current_resolution as i16 - 1 {
                self.current_resolution = gui_app.resolution;
                self.current_bohr = 1.0 / gui_app.size * 1.5;
                self.instance_camera = self.camera;
                self.vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(&voxel::generate_face(self.current_resolution)),
                    usage: wgpu::BufferUsages::VERTEX,});
            }

        //This calls the function to create the instances
        //it spends how many frames it needs to render it all while not causing overhead
            if self.function_index < (LENGTH + 1) * self.current_resolution as i16 {

                let mut instancing_result = function::orbital(self.current_resolution, 1.0 / gui_app.size * 1.5, self.function_index, &self.orbital_array, &self.instance_camera);

                self.new_instances.append(&mut instancing_result.0);
                self.function_index = instancing_result.1;
            }

        //This collects all the instances and makes it so the instance buffer understands it (shader magic, don't ask)
            if self.function_index == (LENGTH + 1) * self.current_resolution as i16 + 1 {

                self.instance_data = self.new_instances.iter().map(instance::Instance::to_raw).collect::<Vec<_>>();
            }

        //Commits all the previously collected data to the instance buffer, ready to be rendered
            if self.function_index == (LENGTH + 1) * self.current_resolution as i16 + 1 {

                self.instances.clear();
                self.instances.append(&mut self.new_instances);

                self.instance_buffer = self.device.create_buffer_init(&BufferInitDescriptor {
                    label: Some("Instance Buffer"),
                    contents: bytemuck::cast_slice(&self.instance_data) ,
                    usage: wgpu::BufferUsages::VERTEX,
                });
            }

        //Good ol' variable reset - makes the loop start again
            if self.function_index == (LENGTH + 1) * self.current_resolution as i16 + 1 {self.function_index = (-LENGTH) * self.current_resolution as i16 - 1;}

        //This is here to prevent that the code instances the last pass and collects the data in the same frame
        //Basically to make it so it runs more consistently
            if self.function_index == (LENGTH + 1) * self.current_resolution as i16 {
                self.function_index += 1;
            }
    }

//RENDER FUNCTION – Renders all the viewport
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
        let command_buffer = (encoder).finish();

        self.queue.submit([command_buffer]);

        output.present();

        Ok(())
    }
}

struct GuiState {
    surface: Surface,
    device: egui_wgpu::wgpu::Device,
    queue: Queue,
    config: SurfaceConfiguration,
    size: PhysicalSize<u32>,
    platform: Platform,
}

impl GuiState {
    async fn gui_new(gui_window: &Window) -> Self {

        let size = gui_window.inner_size();

        let instance = egui_wgpu::wgpu::Instance::new(Backends::all());

        let surface = unsafe {instance.create_surface(gui_window)};

        let adapter = pollster::block_on(instance.request_adapter(
            &egui_wgpu::wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        )).unwrap();

        let (device, queue) = adapter.request_device(
            &egui_wgpu::wgpu::DeviceDescriptor{
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
        };
        surface.configure(&device, &config);

        let platform = Platform::new(egui_winit_platform::PlatformDescriptor {
            physical_width: size.width as u32,
            physical_height: size.height as u32,
            scale_factor: gui_window.scale_factor(),
            font_definitions: FontDefinitions::default(),
            style: Default::default(),
        });

        Self {
            surface,
            device,
            queue,
            config,
            size,
            platform,
        }
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

        window.set_min_inner_size(Some(PhysicalSize::new  (MonitorHandle::size(&window.current_monitor().unwrap()).height / 3 * 2,
        MonitorHandle::size(&window.current_monitor().unwrap()).height / 3 * 2)));

        let gui_window = WindowBuilder::new()
            .with_title("Interface").with_resizable(false).with_always_on_top(true).with_decorations(true)
            .with_inner_size(PhysicalSize::new  (MonitorHandle::size(&window.current_monitor().unwrap()).height * 4 / 9,
                                                MonitorHandle::size(&window.current_monitor().unwrap()).height / 3 * 2))
                                                .with_position(PhysicalPosition::new(MonitorHandle::size(&window.current_monitor().unwrap()).width / 10 * 7,
                                                MonitorHandle::size(&window.current_monitor().unwrap()).height / 6))
            .build(&event_loop)
            .unwrap();

        let mut state = State::new(&window).await;

        let mut gui_state = GuiState::gui_new(&gui_window).await;

        let mut last_render_time = instant::Instant::now();

        let mut egui_rpass = egui_wgpu::renderer::RenderPass::new(&gui_state.device, gui_state.config.format, 1);
        let mut gui_app = interface::Guindow::new(&gui_window);

        state.function_index = -LENGTH * state.current_resolution as i16;

        let start_time = instant::Instant::now();

        event_loop.run(move |event, _, control_flow| {

            gui_state.platform.handle_event(&event);

            match event {
                Event::MainEventsCleared => {window.request_redraw(); gui_window.request_redraw()},

                Event::DeviceEvent {event: DeviceEvent::MouseMotion {delta}, ..} => {
                    state.camera_controller.process_mouse(delta.0, delta.1);
                }

                Event::WindowEvent { window_id, ref event} 
                if window_id == gui_window.id()=> {
                    match event {
                        WindowEvent::CursorMoved {..} => {
                           //gui_window.drag_window().unwrap();
                        }
                        WindowEvent::CursorEntered {..} => {
                            gui_app.enabled = true
                        }
                        WindowEvent::CursorLeft {..} => {
                            gui_app.enabled = false
                        }
                        _ => {}
                    }
                }

                Event::WindowEvent {ref event, window_id,}
                if window_id == window.id() && !state.input(event) => {
                    match event {
                        WindowEvent::CloseRequested | WindowEvent::KeyboardInput {
                            input: KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                        } => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(physical_size) => {
                            if window_id == window.id() {state.resize(*physical_size);} else {}
                        }
                        WindowEvent::ScaleFactorChanged {new_inner_size, .. } => {
                            if window_id == window.id() {state.resize(**new_inner_size);} else {}
                        }
                        _ => {}
                    }
                }

                Event::RedrawRequested(window_id) if window_id == window.id() => {

                    match state.render() {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => state.resize(state.size),
                        Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                        Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                    }

                    let now = instant::Instant::now();
                    let dt = now - last_render_time;
                    last_render_time = now;

                    state.update(&gui_app, dt.as_secs_f32());
                    log::log!(log::Level::Info,"{:?}", state.camera.position);
                }

                Event::RedrawRequested(window_id) if window_id == gui_window.id() => {

                    gui_state.platform.update_time(start_time.elapsed().as_secs_f64());

                    let gui_output = match gui_state.surface.get_current_texture() {
                        Ok(frame) => frame,
                        Err(_) => return,
                    };

                    let gui_view = gui_output
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                    gui_state.platform.begin_frame();

                    gui_app.show(&gui_state.platform.context());

                    let full_output = gui_state.platform.end_frame(Some(&gui_window));

                    let paint_jobs = gui_state.platform.context().tessellate(full_output.shapes);

                    let mut gui_encoder = gui_state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("encoder"),});

                    let screen_descriptor = ScreenDescriptor {
                        size_in_pixels: [gui_state.size.width, gui_state.size.height],
                        pixels_per_point: gui_window.scale_factor() as f32,

                    };

                    let tdelta = full_output.textures_delta.set;

                    let idelta: (Vec<TextureId>, Vec<ImageDelta>) = tdelta.into_iter().unzip();

                    if idelta.0.len() != 0 {
                    egui_rpass.update_texture(&gui_state.device, &gui_state.queue, idelta.0[0],&idelta.1[0]);
                    }

                    egui_rpass.update_buffers(&gui_state.device, &gui_state.queue, &paint_jobs, &screen_descriptor);

                    egui_rpass.execute(&mut gui_encoder, &gui_view, &paint_jobs, &screen_descriptor, Some(wgpu::Color::TRANSPARENT));

                    gui_state.queue.submit(std::iter::once(gui_encoder.finish()));

                    gui_output.present();
                }

                _ => {}
            }
        });
    }
