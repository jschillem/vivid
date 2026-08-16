use log::{debug, info, trace};
use pollster::FutureExt;
use thiserror::Error;
use vivid_core::glam::{Vec2, Vec3};
use wgpu::util::DeviceExt;

use crate::{
    buffer::GpuVec,
    camera::Camera,
    vertex::{Instance, Vertex},
};

const INITIAL_INSTANCE_CAPACITY: u64 = 64;

/// Includes a Slang shader that was compiled to WGSL by `build.rs`.
///
/// ```ignore
/// let shader = device.create_shader_module(include_slang!("triangle"));
/// ```
///
/// You can also override the label shown in error messages/tooling:
///
/// ```ignore
/// let shader = device.create_shader_module(include_slang!("triangle", label: "my triangle"));
/// ```
macro_rules! include_slang {
    ($name:literal) => {
        include_slang!($name, label: $name)
    };
    ($name:literal, label: $label:expr) => {
        wgpu::ShaderModuleDescriptor {
            label: Some($label),
            source: wgpu::ShaderSource::Wgsl(
                include_str!(concat!(env!("OUT_DIR"), "/", $name, ".wgsl")).into(),
            ),
        }
    };
}

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("surface creation failed: {0}")]
    CreateSurface(#[from] wgpu::CreateSurfaceError),
    #[error("no suitable GPU adapter: {0}")]
    RequestAdapter(#[from] wgpu::RequestAdapterError),
    #[error("device request failed: {0}")]
    RequestDevice(#[from] wgpu::RequestDeviceError),
    #[error("GPU device lost")]
    DeviceLost,
}

/// The engine renderer.
#[derive(Debug)]
pub struct Renderer<'window> {
    surface: wgpu::Surface<'window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    configured: bool,
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
    instances: GpuVec<Instance>,
    pub camera: Camera,
    pub clear_color: wgpu::Color,
}

impl<'window> Renderer<'window> {
    /// Create the GPU connection and surface for `target`.
    pub fn new(
        target: impl Into<wgpu::SurfaceTarget<'window>>,
        width: u32,
        height: u32,
    ) -> Result<Self, RenderError> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            // PRIMARY = Vulkan on Linux (+ Metal/DX12 elsewhere).
            backends: wgpu::Backends::PRIMARY,
            flags: wgpu::InstanceFlags::from_env_or_default(),
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
            backend_options: wgpu::BackendOptions::default(),
            display: None,
        });

        let surface = instance.create_surface(target)?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                apply_limit_buckets: true,
            })
            .block_on()?;
        let adapter_info = adapter.get_info();
        info!(
            "adapter: {} ({:?})",
            adapter_info.name, adapter_info.backend
        );

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("vivid-device"),
                required_features: wgpu::Features::empty(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
                trace: wgpu::Trace::Off,
            })
            .block_on()?;

        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(wgpu::TextureFormat::is_srgb)
            .unwrap_or(caps.formats[0]);
        debug!("surface format: {format:?}");

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width,
            height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::default(),
            view_formats: Vec::new(),
            desired_maximum_frame_latency: 2,
            color_space: wgpu::SurfaceColorSpace::Auto,
        };

        let shader = device.create_shader_module(include_slang!("quad"));

        let camera = Camera::new(&device);

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("vivid-pipeline-layout"),
            bind_group_layouts: &[Some(camera.layout())],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("vivid-pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(Vertex::LAYOUT), Some(Instance::LAYOUT)],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
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
            multiview_mask: None,
            cache: None,
        });

        let (vertices, indices) = crate::vertex::cube();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("quad-vertices"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("quad-indices"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let index_count = u32::try_from(indices.len()).expect("index count fits u32");

        let instances = GpuVec::new(
            &device,
            Some("instances"),
            wgpu::BufferUsages::VERTEX,
            INITIAL_INSTANCE_CAPACITY,
        );

        let mut renderer = Self {
            surface,
            device,
            queue,
            config,
            configured: false,
            pipeline,
            vertex_buffer,
            index_buffer,
            index_count,
            instances,
            camera,
            clear_color: wgpu::Color {
                r: 0.06,
                g: 0.06,
                b: 0.06,
                a: 1.0,
            },
        };

        renderer.resize(width, height);
        Ok(renderer)
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            self.configured = false;
            return;
        }

        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        self.configured = true;
        trace!("surface configured at {width}x{height}");
    }

    pub fn ensure_size(&mut self, width: u32, height: u32) {
        if self.configured && width == self.config.width && height == self.config.height {
            return;
        }
        self.resize(width, height);
    }

    pub fn render(&mut self, cubes: &[Vec3]) -> Result<(), RenderError> {
        if !self.configured {
            return Ok(());
        }

        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(t)
            | wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
            wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Occluded
            | wgpu::CurrentSurfaceTexture::Validation => {
                trace!("frame skipped (surface unavailable)");
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.config);
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Lost => return Err(RenderError::DeviceLost),
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        #[allow(
            clippy::cast_precision_loss,
            reason = "23 bits is PLENTY for a window width/height, cast should never fail."
        )]
        self.camera.upload(
            &self.queue,
            self.config.width as f32 / self.config.height as f32,
        );

        self.instances
            .write(&self.device, &self.queue, bytemuck::cast_slice(cubes));

        let mut encoder =
            self.device
                .create_command_encoder(&wgpu::wgt::CommandEncoderDescriptor {
                    label: Some("vivid-frame"),
                });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("main-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            pass.set_pipeline(&self.pipeline);
            pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            pass.set_vertex_buffer(1, self.instances.slice(..));
            pass.set_bind_group(0, self.camera.bind_group(), &[]);
            pass.draw_indexed(0..self.index_count, 0, 0..self.instances.len());
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        self.queue.present(frame);
        Ok(())
    }
}
