use anyhow::anyhow;
use anyhow::Result;
use bytemuck::Pod;
use bytemuck::Zeroable;
use log::warn;
use mappy::Map;
use mappy::SurfaceInfo;
use nalgebra::Point3;
use wgpu::Backends;
use wgpu::BindGroup;
use wgpu::BindGroupDescriptor;
use wgpu::BindGroupEntry;
use wgpu::BindGroupLayout;
use wgpu::BindGroupLayoutDescriptor;
use wgpu::BindGroupLayoutEntry;
use wgpu::BindingResource;
use wgpu::BindingType;
use wgpu::BlendState;
use wgpu::Buffer;
use wgpu::BufferAddress;
use wgpu::BufferBinding;
use wgpu::BufferBindingType;
use wgpu::BufferDescriptor;
use wgpu::BufferSize;
use wgpu::BufferUsages;
use wgpu::Color;
use wgpu::ColorTargetState;
use wgpu::ColorWrites;
use wgpu::CommandEncoderDescriptor;
use wgpu::CompareFunction;
use wgpu::DepthBiasState;
use wgpu::DepthStencilState;
use wgpu::Device;
use wgpu::DeviceDescriptor;
use wgpu::Extent3d;
use wgpu::Features;
use wgpu::FragmentState;
use wgpu::FrontFace;
use wgpu::Instance;
use wgpu::Limits;
use wgpu::LoadOp;
use wgpu::MultisampleState;
use wgpu::Operations;
use wgpu::PipelineLayout;
use wgpu::PipelineLayoutDescriptor;
use wgpu::PolygonMode;
use wgpu::PowerPreference;
use wgpu::PresentMode;
use wgpu::PrimitiveState;
use wgpu::PrimitiveTopology;
use wgpu::Queue;
use wgpu::RenderPassColorAttachment;
use wgpu::RenderPassDepthStencilAttachment;
use wgpu::RenderPassDescriptor;
use wgpu::RenderPipeline;
use wgpu::RenderPipelineDescriptor;
use wgpu::RequestAdapterOptions;
use wgpu::ShaderModule;
use wgpu::ShaderStages;
use wgpu::StencilState;
use wgpu::Surface;
use wgpu::SurfaceConfiguration;
use wgpu::SurfaceError;
use wgpu::Texture;
use wgpu::TextureDescriptor;
use wgpu::TextureDimension;
use wgpu::TextureFormat;
use wgpu::TextureUsages;
use wgpu::TextureView;
use wgpu::TextureViewDescriptor;
use wgpu::VertexBufferLayout;
use wgpu::VertexState;
use wgpu::VertexStepMode;
use wgpu::include_wgsl;
use wgpu::util::BufferInitDescriptor;
use wgpu::util::DeviceExt;
use wgpu::vertex_attr_array;
use winit::window::Window;

use crate::camera::Camera;
use crate::resolution::Resolution;

pub const DEPTH_FORMAT: TextureFormat = TextureFormat::Depth32Float;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Globals {
        proj: [[f32; 4]; 4],
        proj_inv: [[f32; 4]; 4],
        view: [[f32; 4]; 4],
        view_proj: [[f32; 4]; 4],
        cam_pos: [f32; 4],
}

impl Globals {
    fn from_camera(camera: Camera, resolution: Resolution) -> Self {
        let projection = camera.projection(resolution);
        let view = camera.view().to_homogeneous();

        Self {
            proj: projection.into(),
            proj_inv: projection.try_inverse().unwrap().into(),
            view: view.into(),
            view_proj: (projection * view).into(),
            cam_pos: camera.position.to_homogeneous().into(),
        }
    }
}

pub struct Renderer {
    pub surface: Surface,
    pub device: Device,
    pub queue: Queue,
    pub surface_configuration: SurfaceConfiguration,
    pub bind_group_layout: BindGroupLayout,
    pub shader: ShaderModule,
    pub pipeline_layout: PipelineLayout,
    pub pipeline: RenderPipeline,
    pub vertex_counts: Vec<u32>,
    pub vertices: Buffer,
    pub globals: Buffer,
    pub locals: Buffer,
    pub bind_group: BindGroup,
    pub depth_stencil: Texture,
    pub depth_stencil_view: TextureView,
}

impl Renderer {
    pub async fn new(window: &Window, resolution: Resolution, map: &Map<'_>) -> Result<Self> {
        let instance = Instance::new(Backends::PRIMARY);

        let surface = unsafe { instance.create_surface(&window) };

        let adapter = match instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }).await {
            Some(adapter) => adapter,
            None => return Err(anyhow!("no suitable graphics adapter")),
        };

        let (device, queue) = match adapter.request_device(
            &DeviceDescriptor { label: Some("device"), features: Features::empty(), limits: Limits::default() },
            None,
        ).await {
            Ok(dq) => dq,
            Err(_) => return Err(anyhow!("no suitable graphics device")),
        };

        let surface_configuration = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: match surface.get_preferred_format(&adapter) {
                Some(format) => format,
                None => return Err(anyhow!("incompatible surface")),
            },
            width: resolution.width,
            height: resolution.height,
            present_mode: PresentMode::Mailbox,
        };
        surface.configure(&device, &surface_configuration);

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("bind_group_layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: BufferSize::new(std::mem::size_of::<Globals>() as _),
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: BufferSize::new(std::mem::size_of::<SurfaceInfo>() as _),
                    },
                    count: None,
                }
            ],
        });

        let shader = device.create_shader_module(&include_wgsl!("map.wgsl"));

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[VertexBufferLayout {
                    array_stride: std::mem::size_of::<Point3<f32>>() as BufferAddress,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &vertex_attr_array![0 => Float32x3],
                }],
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: CompareFunction::GreaterEqual,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            multisample: MultisampleState::default(),
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[ColorTargetState {
                    format: surface_configuration.format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                }],
            }),
            multiview: None,
        });

        let vertices = device.create_buffer(&BufferDescriptor {
            label: Some("vertices"),
            size: 0,
            usage: BufferUsages::COPY_DST | BufferUsages::VERTEX,
            mapped_at_creation: false,
        });

        let globals = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("globals"),
            contents: bytemuck::bytes_of(&Globals::from_camera(Camera::default(), resolution)),
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
        });

        let locals = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("locals"),
            contents: bytemuck::cast_slice(&map.surface_info),
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("bind_group"),
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: globals.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Buffer(BufferBinding {
                        buffer: &locals,
                        offset: 0,
                        size: BufferSize::new(std::mem::size_of::<SurfaceInfo>() as u64),
                    })
                }
            ],
        });

        let (depth_stencil, depth_stencil_view) = create_depth_stencil(&device, resolution);

        Ok(Self {
            surface,
            device,
            queue,
            surface_configuration,
            bind_group_layout,
            shader,
            pipeline_layout,
            pipeline,
            vertex_counts: vec![],
            vertices,
            globals,
            locals,
            bind_group,
            depth_stencil,
            depth_stencil_view,
        })
    }

    pub fn resize(&mut self, resolution: Resolution) {
        if resolution.width == 0 || resolution.height == 0 { return; }

        self.surface_configuration.width = resolution.width;
        self.surface_configuration.height = resolution.height;
        self.surface.configure(&self.device, &self.surface_configuration);

        let (depth_stencil, depth_stencil_view) = create_depth_stencil(&self.device, resolution);
        self.depth_stencil = depth_stencil;
        self.depth_stencil_view = depth_stencil_view;
    }

    pub fn write_globals(&self, camera: Camera, resolution: Resolution) {
        self.queue.write_buffer(&self.globals, 0, bytemuck::bytes_of(&Globals::from_camera(camera, resolution)));
    }

    pub fn load_map(&mut self, map: &Map) {
        self.vertex_counts = map.vertex_counts.to_owned();
        self.vertices = self.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("vertices"),
            contents: bytemuck::cast_slice(&map.vertices),
            usage: BufferUsages::COPY_DST | BufferUsages::VERTEX,
        });
    }

    pub fn render(&self) -> Result<()> {
        let surface = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(e) => match e {
                SurfaceError::Timeout => {
                    warn!("Timed out while retrieving surface");
                    return Ok(());
                },
                SurfaceError::Outdated => {
                    warn!("Retrieved surface was outdated");
                    return Ok(());
                },
                SurfaceError::Lost => return Err(anyhow!("surface lost")),
                SurfaceError::OutOfMemory => return Err(anyhow!("ran out of memory while retrieving surface")),
            },
        };

        let surface_view = surface.texture.create_view(&TextureViewDescriptor::default());

        let mut command_encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("command_encoder"),
        });

        {
            let mut render_pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("render_pass"),
                color_attachments: &[RenderPassColorAttachment {
                    view: &surface_view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &self.depth_stencil_view,
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(0.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_vertex_buffer(0, self.vertices.slice(..));
            let mut start = 0;
            for i in 0..self.vertex_counts.len() {
                render_pass.set_bind_group(0, &self.bind_group, &[(i * std::mem::size_of::<SurfaceInfo>()) as u32]);
                let vertex_count = self.vertex_counts[i];
                render_pass.draw(start..start + vertex_count, 0..1);
                start += vertex_count;
            }
        }

        self.queue.submit([command_encoder.finish()]);
        surface.present();
        Ok(())
    }
}

fn create_depth_stencil(device: &Device, resolution: Resolution) -> (Texture, TextureView) {
    let depth_stencil = device.create_texture(&TextureDescriptor {
        label: Some("depth_stencil"),
        size: Extent3d {
            width: resolution.width,
            height: resolution.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: DEPTH_FORMAT,
        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
    });

    let depth_stencil_view = depth_stencil.create_view(&TextureViewDescriptor::default());

    (depth_stencil, depth_stencil_view)
}
