use bytemuck::Pod;
use bytemuck::Zeroable;
use mappy::Map;
use mappy::SurfaceInfo;
use nalgebra::Point3;
use rendering_util::RenderingContext;
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
use wgpu::Extent3d;
use wgpu::FragmentState;
use wgpu::FrontFace;
use wgpu::LoadOp;
use wgpu::MultisampleState;
use wgpu::Operations;
use wgpu::PipelineLayout;
use wgpu::PipelineLayoutDescriptor;
use wgpu::PolygonMode;
use wgpu::PrimitiveState;
use wgpu::PrimitiveTopology;
use wgpu::RenderPassColorAttachment;
use wgpu::RenderPassDepthStencilAttachment;
use wgpu::RenderPassDescriptor;
use wgpu::RenderPipeline;
use wgpu::RenderPipelineDescriptor;
use wgpu::ShaderModule;
use wgpu::ShaderStages;
use wgpu::StencilState;
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

use crate::camera::Camera;

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
    fn from_camera(camera: &Camera, width: u32, height: u32) -> Self {
        let projection = camera.projection(width, height);
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

#[allow(dead_code)]
pub struct MapRenderer {
    width: u32,
    height: u32,
    bind_group_layout: BindGroupLayout,
    shader: ShaderModule,
    pipeline_layout: PipelineLayout,
    pipeline: RenderPipeline,
    vertex_counts: Vec<u32>,
    vertices: Buffer,
    globals: Buffer,
    locals: Buffer,
    bind_group: BindGroup,
    depth_stencil: Texture,
    depth_stencil_view: TextureView,
}

impl MapRenderer {
    pub fn new(rc: &RenderingContext, map: &Map<'_>) -> Self {
        let width = rc.width();
        let height = rc.height();

        let bind_group_layout = rc.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
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

        let shader = rc.device.create_shader_module(&include_wgsl!("map.wgsl"));

        let pipeline_layout = rc.device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = rc.device.create_render_pipeline(&RenderPipelineDescriptor {
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
                    format: rc.surface_format(),
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                }],
            }),
            multiview: None,
        });

        let vertices = rc.device.create_buffer(&BufferDescriptor {
            label: Some("vertices"),
            size: 0,
            usage: BufferUsages::COPY_DST | BufferUsages::VERTEX,
            mapped_at_creation: false,
        });

        let globals = rc.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("globals"),
            contents: bytemuck::bytes_of(
                &Globals::from_camera(&Camera::default(), width, height)
            ),
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
        });

        let locals = rc.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("locals"),
            contents: bytemuck::cast_slice(&map.surface_info),
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
        });

        let bind_group = rc.device.create_bind_group(&BindGroupDescriptor {
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

        let (depth_stencil, depth_stencil_view) = create_depth_stencil(&rc.device, rc.width(), rc.height());

        Self {
            height,
            width,
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
        }
    }

    pub fn load_map(&mut self, rc: &RenderingContext, map: &Map) {
        self.vertex_counts = map.vertex_counts.to_owned();
        self.vertices = rc.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("vertices"),
            contents: bytemuck::cast_slice(&map.vertices),
            usage: BufferUsages::COPY_DST | BufferUsages::VERTEX,
        });
    }

    pub fn render(&mut self, rc: &RenderingContext, surface_view: &TextureView, camera: &Camera) {
        // Update resolution if it's dirty
        if self.width != rc.width() || self.height != rc.height() {
            self.width = rc.width();
            self.height = rc.height();
            let (ds, dsv) = create_depth_stencil(&rc.device, self.width, self.height);
            self.depth_stencil = ds;
            self.depth_stencil_view = dsv;
        }

        // Write our globals
        rc.queue.write_buffer(
            &self.globals,
            0,
            bytemuck::bytes_of(&Globals::from_camera(camera, self.width, self.height),
        ));

        // Build our command encoder
        let mut command_encoder = rc.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("command_encoder"),
        });

        // Render it!
        {
            let mut render_pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("render_pass"),
                color_attachments: &[RenderPassColorAttachment {
                    view: surface_view,
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

        // Submit our work
        rc.queue.submit([command_encoder.finish()]);
    }
}

fn create_depth_stencil(device: &Device, width: u32, height: u32) -> (Texture, TextureView) {
    let depth_stencil = device.create_texture(&TextureDescriptor {
        label: Some("depth_stencil"),
        size: Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: DEPTH_FORMAT,
        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
    });

    let depth_stencil_view = depth_stencil.create_view(&TextureViewDescriptor::default());

    (depth_stencil, depth_stencil_view)
}
