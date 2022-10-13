mod geometry;
mod uniform;
mod time;
mod camera;
mod texture;

use std::{borrow::Cow, iter};
use wgpu::*;
use winit::{dpi::PhysicalSize, event::*, window::Window};
use time::Instant;
use self::{
    camera::{Camera, Projection, controller::CameraController},
    geometry::{Vertex, VertexBinding, quad::{QuadVertex, DrawQuad}},
    uniform::{Uniform, UniformBinding, system::SystemUniform, camera::CameraUniform}
};
#[cfg(target_arch="wasm32")]
use wasm_bindgen::prelude::*;

fn compile_shader(device: &Device, source: &str) -> ShaderModule {
    device.create_shader_module(ShaderModuleDescriptor {
        label: Some("Shader"),
        source: ShaderSource::Wgsl(Cow::Borrowed(source))
    })
}

fn create_pipeline(device: &Device, layout: &PipelineLayout, color_format: TextureFormat, depth_format: Option<TextureFormat>, vertex_layouts: &[VertexBufferLayout], shader: ShaderModule) -> RenderPipeline {
    device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(layout),
        // vertex shader and buffers
        vertex: VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: vertex_layouts,
        },
        // fragment shader and buffers and blending modes
        fragment: Some(FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(ColorTargetState {
                format: color_format,
                blend: Some(BlendState {
                    alpha: BlendComponent::REPLACE,
                    color: BlendComponent::REPLACE,
                }),
                write_mask: ColorWrites::ALL,
            })],
        }),
        // how to interpret vertices as triangles
        primitive: PrimitiveState {
            topology: PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: FrontFace::Ccw,
            cull_mode: Some(Face::Back),
            // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE or Features::POLYGON_MODE_POINT
            polygon_mode: PolygonMode::Fill,
            // Requires Features::DEPTH_CLIP_CONTROL
            unclipped_depth: false,
            // Requires Features::CONSERVATIVE_RASTERIZATION
            conservative: false,
        },
        depth_stencil: depth_format.map(|format| DepthStencilState {
            format,
            depth_write_enabled: true,
            depth_compare: CompareFunction::Less,
            stencil: StencilState::default(),
            bias: DepthBiasState::default(),
        }),
        multisample: MultisampleState {
            count: 1,
            mask: !0,
            // No antialiasing
            alpha_to_coverage_enabled: false,
        },
        // If the pipeline will be used with a multiview render pass, this indicates how many array layers the attachments will have.
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
    mesh: VertexBinding,
    /* binding */
    size: PhysicalSize<u32>,
    start_render_time: Instant,
    last_render_time: Instant,
    mouse_lock: bool,
    camera: Camera,
    projection: Projection,
    camera_controller: CameraController,
    system_uniform: UniformBinding<SystemUniform>,
    camera_uniform: UniformBinding<CameraUniform>,
}

#[cfg_attr(target_arch="wasm32", wasm_bindgen)]
impl State {
    pub(super) async fn new(source: &str, window: &Window) -> Self {
        let size = window.inner_size();
        let start_render_time = Instant::now();
        let last_render_time = Instant::now();
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
                limits: if cfg!(target_arch = "wasm32") { Limits::downlevel_webgl2_defaults() } else { Limits::default() },
            },
            None, // Trace path
        ).await.unwrap();

        let surface_config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Fifo,
            alpha_mode: CompositeAlphaMode::Auto,
        };
        surface.configure(&device, &surface_config);

        // SYSTEM BINDING
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
        let system_uniform = SystemUniform::new(size, start_render_time).make_binding(&device, &system_bind_group_layout);

        // CAMERA BINDING
        let camera_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Camera Buffer Bind Group Layout"),
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
        let camera_uniform = CameraUniform::new().make_binding(&device, &camera_bind_group_layout);

        let shader = compile_shader(&device, source);
        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&system_bind_group_layout, &camera_bind_group_layout],
            push_constant_ranges: &[],
        });
        let render_pipeline = create_pipeline(&device, &render_pipeline_layout, surface_config.format, None, &[QuadVertex::desc()], shader);

        let mesh = QuadVertex::new().make_binding(&device);

        let camera = Camera::new((0.0, 5.0, 10.0), cgmath::Deg(-90.0), cgmath::Deg(-20.0));
        let projection = Projection::new(size.width, size.height, cgmath::Deg(45.0), 0.1, 100.0);
        let camera_controller = CameraController::new(4.0, 0.4);

        Self {
            surface, device, queue, surface_config, render_pipeline, render_pipeline_layout,
            mesh, size, start_render_time, last_render_time, mouse_lock: false,
            camera, projection, camera_controller, system_uniform, camera_uniform
        }
    }

    pub(super) fn recompile(&mut self, source: &str) {
        self.render_pipeline = create_pipeline(&self.device, &self.render_pipeline_layout, self.surface_config.format, None,
                                               &[QuadVertex::desc()], compile_shader(&self.device, source))
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

    pub(super) fn handle_mouse_input(&mut self, x: f64, y: f64) {
        if self.mouse_lock {
            let _ = &self.camera_controller.process_mouse(x, y);
        }
    }

    pub(super) fn handle_input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                KeyboardInput {
                    virtual_keycode: Some(key),
                    state,
                    ..
                },
                ..
            } => self.camera_controller.process_keyboard(*key, *state),
            WindowEvent::MouseWheel { delta, .. } => {
                self.camera_controller.process_scroll(delta);
                true
            }
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => {
                self.mouse_lock = *state == ElementState::Pressed;
                true
            }
            _ => false,
        }
    }

    pub(super) fn update(&mut self) {
        self.camera_controller.update_camera(&mut self.camera, Instant::now() - self.last_render_time);
        self.camera_uniform.uniform_mut().update_view_proj(&self.camera, &self.projection);
        self.system_uniform.uniform_mut().update_system(self.size, self.start_render_time);
        self.queue.write_buffer(self.system_uniform.buffer(), 0, bytemuck::cast_slice(&[*self.system_uniform.uniform()]));
        self.queue.write_buffer(self.camera_uniform.buffer(), 0, bytemuck::cast_slice(&[*self.camera_uniform.uniform()]));
    }

    pub(super) fn render(&mut self) -> Result<(), SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor { label: Some("Render Encoder") });

        encoder.push_debug_group("rendering passes");
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
            render_pass.set_bind_group(1, self.camera_uniform.bind_group(), &[]);
            render_pass.draw_mesh(&self.mesh);
        }
        encoder.pop_debug_group();

        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
