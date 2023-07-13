
use text::CharacterQuad;
use wgpu::Label;
use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder, self},
};
use cgmath::prelude::*;


mod text;
mod texture;
mod resources;

fn main() {
    pollster::block_on(run());
}
async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut state = State::new(window).await;

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() => if !state.input(event) { 
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
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
                state.update();
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
    t_render_pipeline: wgpu::RenderPipeline,
    t_texture: texture::Texture,
    t_vertices: wgpu::Buffer,
    t_indices: wgpu::Buffer,
    t_bind_group: wgpu::BindGroup,
    test_len: u32,
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

        // Test Rendering Testing
        // ----------------------

        /*let scale: f32 = 1.0 / 9.0;

        let x: f32 = 2.0;
        let y: f32 = 1.0;

        let tl = [x * scale, y * scale];
        let bl = [x * scale, y * scale + scale];
        let br = [x * scale + scale, y * scale + scale];
        let tr = [x * scale + scale, y * scale];

        let test_vertex = &[
            text::CharacterVertex { position: [-0.5, 0.5, 0.0], tex_coords: tl }, //[0.0, 0.0] }, // TOP LEFT
            text::CharacterVertex { position: [-0.5, -0.5, 0.0], tex_coords: bl }, //[0.0, 1.0/16.0] }, // BOTTOM LEFT
            text::CharacterVertex { position: [0.5, -0.5, 0.0], tex_coords: br }, //[1.0/16.0, 1.0/16.0] }, // BOTTOM RIGHT
            text::CharacterVertex { position: [0.5, 0.5, 0.0], tex_coords: tr }, //[1.0/16.0, 0.0] }, // TOP RIGHT
        
            text::CharacterVertex { position: [-0.25, 0.5, 0.0], tex_coords: tl }, // TOP LEFT
            text::CharacterVertex { position: [-0.25, -0.5, 0.0], tex_coords: bl }, // BOTTOM LEFT
            text::CharacterVertex { position: [0.75, -0.5, 0.0], tex_coords: br }, // BOTTOM RIGHT
            text::CharacterVertex { position: [0.75, 0.5, 0.0], tex_coords: tr }, // TOP RIGHT
        ];

        /*let test_vertex = &[
            text::CharacterVertex { position: [0.0, 0.5, 0.0], tex_coords: [0.0, 0.0] },
            text::CharacterVertex { position: [-0.5, -0.5, 0.0], tex_coords: [0.0, 0.0] },
            text::CharacterVertex { position: [0.5, -0.5, 0.0], tex_coords: [0.0, 0.0] },
        ];*/

        let test_indices = &[
            1, 3, 0, 1, 2, 3,
            5, 7, 4, 5, 6, 7
        ];

        let test_len = test_indices.len() as u32;

        let t_vertices = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(test_vertex),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let t_indices = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(test_indices),
                usage: wgpu::BufferUsages::INDEX,
            }
        );*/

        let test_quad = text::CharacterQuad {
            position: [-0.5, 0.5, 0.0],
            size: [0.1, 0.1],
            character: 6,
        };
        let test_quad2 = text::CharacterQuad {
            position: [-0.4, 0.5, 0.0],
            size: [0.1, 0.1],
            character: 10,
        };

        let test_str = "Hello World";

        //let test_vecs = text::TextVecs::from_quad(test_quad, None);
        //let test_vecs = text::TextVecs::from_quads(vec![test_quad, test_quad2]);
        let test_quads = text::character_quads_from_str(test_str);
        let test_vecs = text::TextVecs::from_quads(test_quads);

        let test_bufs = test_vecs.to_buffers(&device);

        let (t_vertices, t_indices , test_len) = 
            (test_bufs.vertices, test_bufs.indices, test_bufs.length);

        let t_texture =
            resources::load_texture("texture1_letters.png", &device, &queue)
            .await
            .unwrap();

        let t_texture_bind_group_layout = device.create_bind_group_layout(
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
                label: Some("texture_bind_group_layout"),
            }
        );

        let t_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &t_texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&t_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&t_texture.sampler),
                    }
                ],
                label: Some("t_bind_group"),
            }
        );

        let t_render_pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &t_texture_bind_group_layout
                ],
                push_constant_ranges: &[],
            }
        );

        let t_render_pipeline = {
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Text Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("text.wgsl").into()),
            };

            create_render_pipeline(
                &device,
                &t_render_pipeline_layout,
                config.format,
                None,
                &[
                    text::CharacterVertex::desc(), 
                ],
                shader,
            )
        };

        State {
            surface,
            device,
            queue,
            config,
            size,
            window,
            t_render_pipeline,
            t_texture,
            t_vertices,
            t_indices,
            t_bind_group,
            test_len,
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
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {

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

            render_pass.set_pipeline(&self.t_render_pipeline);
            render_pass.set_vertex_buffer(0, self.t_vertices.slice(..));
            render_pass.set_index_buffer(self.t_indices.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.set_bind_group(0, &self.t_bind_group, &[]);
            render_pass.draw_indexed(0..self.test_len, 0, 0..1);

            //render_pass.set_pipeline(&self.t_render_pipeline2);
            //render_pass.set_vertex_buffer(0, self.t_vertices2.slice(..));
            //render_pass.draw(0..1, 0..1);

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
                            alpha: wgpu::BlendComponent::REPLACE,
                            color: wgpu::BlendComponent::REPLACE,
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