

use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::ControlFlow,
    window::Window,
};
use std::{f32::consts::SQRT_2, time};

mod text;
mod texture;
mod resources;
mod camera;
mod input_general;

fn main() {
    pollster::block_on(run());
}

struct UIRenderable;
struct Framerate (u32);

async fn run() {
    env_logger::init();
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new().build(&event_loop).unwrap();

    let mut state = State::new(window).await;

    // Testing out new system

    let a_quad = text::CharacterQuad {
        position: [100.0, 100.0, 0.0],
        size: [50.0, 100.0],
        character: 97,  
    };

    //let a = state.world.spawn((UIRenderable, a_quad));

    let a_text = text::character_quads_from_str("Test", vec![150.0, 150.0, 0.0], 50.0);

    let a = state.world.spawn((UIRenderable, a_text));

    state.ui_changed = true;

    //state.camera.modify_position(10.0, 10.0);
    //state.camera_uniform.update_proj(&state.camera);
    //state.queue.write_buffer(&state.camera_buffer, 0, bytemuck::cast_slice(&[state.camera_uniform]));

    // ----------------------

    let mut last_render_time = time::Instant::now();
    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() => if !state.input(event) { 
                match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    },
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.resize(**new_inner_size);
                    },
                    _ => {},
                }
            },
            Event::RedrawRequested(window_id) if window_id == state.window().id() => {
                let now = time::Instant::now();
                let dt = now - last_render_time;
                last_render_time = now;
                state.input.next_frame();
                state.update(dt);
                match state.render() {
                    Ok(_) => {},
                    Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    Err(e) => eprintln!("{:?}", e),
                }
            },
            Event::MainEventsCleared => {
                state.window().request_redraw();
            },
            _ => {}
        }
    });
}

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: Window,
    world: hecs::World,
    input: input_general::Input,
    ui_layer: text::TextBuffers,
    ui_changed: bool,
    ui_render_pipeline: wgpu::RenderPipeline,
    camera_bind_group: wgpu::BindGroup,
    camera: camera::Camera,
    camera_uniform: camera::CameraUniform,
    camera_buffer: wgpu::Buffer,
    ui_bind_group: wgpu::BindGroup,
    ui_texture: texture::Texture,
    framerate_entity: hecs::Entity,
}

impl State {
    async fn new(window: Window) -> Self {
        // Required Initialisation
        // -----------------------
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });
        
        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                label: None,
            },
            None,
        ).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps.formats.iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let mut world = hecs::World::new();

        let input = input_general::Input::default();

        let camera = camera::Camera::new(800.0, 600.0, 0.0, 0.0);

        let mut camera_uniform = camera::CameraUniform::new();
        camera_uniform.update_proj(&camera);

        let camera_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let camera_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
                ],
                label: Some("camera_bind_group_layout"),
            }
        );

        let camera_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor { 
                label: Some("t_camera_bind_group"), 
                layout: &camera_bind_group_layout, 
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: camera_buffer.as_entire_binding(),
                    }
                ],
            }
        );

        let ui_empty = text::TextVecs {vertices: Vec::new(), indices: Vec::new()};
        let ui_layer = ui_empty.to_buffers(&device);
        let ui_changed = false;

        let ui_texture1 = 
            resources::load_texture("texture1_letters.png", &device, &queue)
            .await
            .unwrap();
        let ui_texture = 
            resources::load_texture("all_16x16.png", &device, &queue)
            .await
            .unwrap();

        let ui_texture_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("ui_texture_bind_group_layout"),
            }
        );

        let ui_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &ui_texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&ui_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&ui_texture.sampler),
                    }
                ],
                label: Some("ui_texture_bind_group"),
            }
        );

        let ui_render_pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &ui_texture_bind_group_layout,
                    &camera_bind_group_layout,
                ],
                push_constant_ranges: &[],
            }
        );

        let ui_render_pipeline = {
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Text Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("text.wgsl").into()),
            };

            create_render_pipeline(
                &device,
                &ui_render_pipeline_layout,
                config.format,
                None,
                &[
                    text::CharacterVertex::desc(), 
                ],
                shader,
            )
        };

        let framerate_text = text::character_quads_from_str("0", vec![20.0, 20.0, 0.0], 20.0);
        let framerate_entity = world.spawn((UIRenderable, Framerate(0), framerate_text));

        State {
            surface,
            device,
            queue,
            config,
            size,
            window,
            world,
            input,
            camera,
            ui_layer,
            ui_changed,
            ui_render_pipeline,
            camera_bind_group,
            camera_uniform,
            camera_buffer,
            ui_bind_group,
            ui_texture,
            framerate_entity,
        }
    }

    fn window(&self) -> &Window {
        &self.window
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) { 
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);

            self.camera.width = new_size.width as f32;
            self.camera.height = new_size.height as f32;

            self.camera_uniform.update_proj(&self.camera);
            self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input: key_ev,
                ..
            } => {
                self.input.handle_key_event(*key_ev);
                true
            },
            WindowEvent::MouseInput { state, button, .. } => {
                self.input.handle_mouse_button(*state, *button);
                true
            },
            WindowEvent::CursorMoved {  position, .. } => {
                self.input.handle_mouse_move(*position);
                true
            },
            _ => false
        }
        /*
        With this it is easy to poll specific pressed keys, ie for continuous movement,
        for events on the press just poll the difference between current and old. 
         */
    }

    fn update(&mut self, dt: time::Duration) {
        // display framerate
        let mut elements: Vec<text::CharacterQuad> = Vec::new();
        {
            let millis = if dt.as_millis() == 0 { 1 } else { dt.as_millis() };
            let framerate = (1000 / millis) as u32;
            //let mut framerate_comp = self.world.get::<&mut Framerate>(self.framerate_entity).unwrap();

            //let mut framerate_e = self.world.query_mut::<(&Framerate, &UIRenderable, &Vec<text::CharacterQuad>)>().into_iter().next().unwrap();

            //let prev = self.world.get::<&Framerate>(self.framerate_entity).unwrap();

            let e = self.world.entity(self.framerate_entity).unwrap();

            if framerate != e.get::<&Framerate>().unwrap().0 {
                self.ui_changed = true;
                //framerate_e.1.0 = &Framerate(framerate);
                let framerate_text = text::character_quads_from_str(&framerate.to_string(), vec![20.0, 20.0, 0.0], 20.0);
                //framerate_e.1.2 = &framerate_text;

                

                *e.get::<&mut Framerate>().unwrap() = Framerate(framerate);
                *e.get::<&mut Vec<text::CharacterQuad>>().unwrap() = framerate_text;

                //self.world.exchange::<(Framerate, Vec<text::CharacterQuad>), (Framerate, Vec<text::CharacterQuad>)>(self.framerate_entity, (Framerate(framerate), framerate_text));
            }
        }
        
        if self.ui_changed {

            for (id, (ui, quad)) in self.world.query_mut::<(&UIRenderable, &Vec<text::CharacterQuad>)>() {
                elements.extend(quad);
            }

            self.ui_layer = text::TextVecs::from_quads(&elements).to_buffers(&self.device);

            self.ui_changed = false;
        }

        let speed = 1.0;
        let mut moved = false;
        let (mut x, mut y) = (0.0, 0.0);

        let movement_keys = ( self.input.is_key_down(winit::event::VirtualKeyCode::Left)
                            , self.input.is_key_down(winit::event::VirtualKeyCode::Right)
                            , self.input.is_key_down(winit::event::VirtualKeyCode::Up)
                            , self.input.is_key_down(winit::event::VirtualKeyCode::Down));
        
        match movement_keys {
            (true, false, false, false) | (true, false, true, true) => { // Left
                x = -speed;
                moved = true;
            },
            (false, true, false, false) | (false, true, true, true) => { // Right
                x = speed;
                moved = true;
            },
            (false, false, true, false) | (true, true, true, false) => { // Up
                y = speed;
                moved = true;
            },
            (false, false, false, true) | (true, true, false, true) => { // Down
                y = -speed;
                moved = true;
            },
            (true, false, true, false) => { // Left & Up
                x = -speed * SQRT_2;
                y = speed * SQRT_2;
                moved = true;
            },
            (false, true, true, false) => { // Right & Up
                x = speed * SQRT_2;
                y = speed * SQRT_2;
                moved = true;
            },
            (true, false, false, true) => { // Left & Down
                x = -speed * SQRT_2;
                y = -speed * SQRT_2;
                moved = true;
            },
            (false, true, false, true) => { // Right & Down
                x = speed * SQRT_2;
                y = -speed * SQRT_2;
                moved = true;
            },
            _ => {},
        }


        if moved {
            self.camera.modify_position(x, y);
            self.camera_uniform.update_proj(&self.camera);
            self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));
        }

    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(
                                wgpu::Color {
                                    r: 0.1,
                                    g: 0.2,
                                    b: 0.3,
                                    a: 1.0,
                                }
                            ),
                            store: true,
                        }
                    }
                )],
                depth_stencil_attachment: None,
            });


            render_pass.set_pipeline(&self.ui_render_pipeline);
            render_pass.set_vertex_buffer(0, self.ui_layer.vertices.slice(..));
            render_pass.set_index_buffer(self.ui_layer.indices.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.set_bind_group(0, &self.ui_bind_group, &[]);
            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
            render_pass.draw_indexed(0..self.ui_layer.length, 0, 0..1);


        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();


        Ok(())

    }

}

fn create_render_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    color_format: wgpu::TextureFormat,
    depth_format: Option<wgpu::TextureFormat>,
    vertex_layouts: &[wgpu::VertexBufferLayout],
    shader: wgpu::ShaderModuleDescriptor,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(shader);

    device.create_render_pipeline(
        &wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: vertex_layouts,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[
                    Some(wgpu::ColorTargetState {
                        format: color_format,
                        blend: Some(wgpu::BlendState {
                            alpha: wgpu::BlendComponent::OVER,
                            color: wgpu::BlendComponent::OVER,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })
                ],
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
        }
    )
}