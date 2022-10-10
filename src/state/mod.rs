mod geometry;
mod uniforms;
mod time;

use std::{borrow::Cow, iter};
use wgpu::{util::DeviceExt, *};
use winit::{dpi::PhysicalSize, event::*, window::Window};
use time::Instant;
use self::{
    geometry::{Vertex, INDICES, VERTICES},
    uniforms::{bindings::{Uniform, UniformBinding}, MouseUniform, SystemUniform}
};
#[cfg(target_arch="wasm32")]
use wasm_bindgen::prelude::*;

fn compile_shader(device: &Device, source: &str) -> ShaderModule {
    device.create_shader_module(ShaderModuleDescriptor {
        label: Some("Shader"),
        source: ShaderSource::Wgsl(Cow::Borrowed(source))
    })
}

fn create_pipeline(device: &Device, surface_config: &SurfaceConfiguration, render_pipeline_layout: &PipelineLayout, shader: ShaderModule ) -> RenderPipeline {
    device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(render_pipeline_layout),
        // vertex shader and buffers
        vertex: VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[Vertex::desc()],
        },
        // fragment shader and buffers and blending modes
        fragment: Some(FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(ColorTargetState {
                format: surface_config.format,
                blend: Some(BlendState::REPLACE),
                write_mask: ColorWrites::ALL,
            })],
        }),
        // how to interpret vertices as triangles
        primitive: PrimitiveState {
            topology: PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: FrontFace::Ccw,
            cull_mode: Some(Face::Back),
            // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
            // or Features::POLYGON_MODE_POINT
            polygon_mode: PolygonMode::Fill,
            // Requires Features::DEPTH_CLIP_CONTROL
            unclipped_depth: false,
            // Requires Features::CONSERVATIVE_RASTERIZATION
            conservative: false,
        },
        depth_stencil: None,
        multisample: MultisampleState {
            count: 1,
            mask: !0,
            // No antialiasing
            alpha_to_coverage_enabled: false,
        },
        // If the pipeline will be used with a multiview render pass, this indicates how
        // many array layers the attachments will have.
        multiview: None,
    })
}

pub(super) struct State {
    surface: Surface,
    surface_config: SurfaceConfiguration,
    device: Device,
    queue: Queue,
    render_pipeline: RenderPipeline,
    render_pipeline_layout: PipelineLayout,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    num_indices: u32,
    /* binding */
    size: PhysicalSize<u32>,
    start_time: Instant,
    system_uniform: UniformBinding<SystemUniform>,
    mouse_uniform: UniformBinding<MouseUniform>
}

#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
impl State {
    pub(super) async fn new(source: &str, window: &Window) -> Self {
        let size = window.inner_size();
        let start_time = Instant::now();
        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = Instance::new(Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }).await.unwrap();

        let (device, queue) = adapter.request_device(
            &DeviceDescriptor {
                label: None,
                features: Features::empty(),
                limits: if cfg!(target_arch = "wasm32") {
                    Limits::downlevel_webgl2_defaults()
                } else {
                    Limits::default()
                },
            },
            None, // Trace path
        ).await.unwrap();

        let surface_config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Fifo,
            alpha_mode: CompositeAlphaMode::Auto
        };
        surface.configure(&device, &surface_config);

        // TIME BINDING
        let system_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Time Buffer Bind Group Layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let system_uniform = SystemUniform::new(size, start_time).make_binding(&device, &system_bind_group_layout);

        // MOUSE BINDING
        let mouse_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Mouse Buffer Bind Group Layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let mouse_uniform = MouseUniform::new().make_binding(&device, &mouse_bind_group_layout);

        let shader = compile_shader(&device, source);
        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&system_bind_group_layout, &mouse_bind_group_layout],
            push_constant_ranges: &[],
        });
        let render_pipeline = create_pipeline(&device, &surface_config, &render_pipeline_layout, shader);

        let vertex_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: BufferUsages::INDEX,
        });
        let num_indices = INDICES.len() as u32;

        Self {
            surface, device, queue, surface_config, render_pipeline, render_pipeline_layout,
            vertex_buffer, index_buffer, num_indices,

            start_time, system_uniform, mouse_uniform, size,
        }
    }

    pub(super) fn recompile(&mut self, source: &str) {
        self.render_pipeline = create_pipeline(&self.device, &self.surface_config, &self.render_pipeline_layout, compile_shader(&self.device, source))
    }

    pub(super) fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface.configure(&self.device, &self.surface_config);
        }
    }

    pub(super) fn current_size(&self) -> PhysicalSize<u32> {
        self.size
    }

    pub(super) fn input(&mut self, event: &WindowEvent) -> bool {
        match *event {
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_uniform.uniform_mut().update_position(
                    (position.x / self.size.width as f64) as f32,
                    (position.y / self.size.height as f64) as f32,
                );
                true
            }
            _ => false,
        }
    }

    pub(super) fn update(&mut self) {
        self.system_uniform.uniform_mut().update(self.size, self.start_time);
        self.queue.write_buffer(self.system_uniform.buffer(), 0, bytemuck::cast_slice(&[*self.system_uniform.uniform()]));
        self.queue.write_buffer(self.mouse_uniform.buffer(), 0, bytemuck::cast_slice(&[*self.mouse_uniform.uniform()]));
    }

    pub(super) fn render(&mut self) -> Result<(), SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Render Encoder")
        });

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color { r: 0.1, g: 0.2, b: 0.3, a: 1.0 }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, self.system_uniform.bind_group(), &[]);
            render_pass.set_bind_group(1, self.mouse_uniform.bind_group(), &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);

            // drop render pass (which owns a &mut encoder) so it can be .finish()ed
            //drop(render_pass);
        }

        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}