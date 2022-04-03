use rendering_util::RenderingContext;
use wgpu::BindGroup;
use wgpu::BindGroupDescriptor;
use wgpu::BindGroupEntry;
use wgpu::BindGroupLayout;
use wgpu::BindGroupLayoutDescriptor;
use wgpu::BindGroupLayoutEntry;
use wgpu::BindingType;
use wgpu::BlendState;
use wgpu::Buffer;
use wgpu::BufferBindingType;
use wgpu::BufferDescriptor;
use wgpu::BufferSize;
use wgpu::BufferUsages;
use wgpu::ColorTargetState;
use wgpu::ColorWrites;
use wgpu::CommandEncoderDescriptor;
use wgpu::CompareFunction;
use wgpu::DepthBiasState;
use wgpu::DepthStencilState;
use wgpu::Face;
use wgpu::FragmentState;
use wgpu::FrontFace;
use wgpu::IndexFormat;
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
use wgpu::TextureView;
use wgpu::VertexState;
use wgpu::include_wgsl;

use super::DEPTH_FORMAT;
use super::Globals;
use super::Instance;
use super::Model;
use super::Vertex;

#[allow(dead_code)]
pub struct ModelRenderer {
    shader: ShaderModule,
    bind_group_layout: BindGroupLayout,
    pipeline_layout: PipelineLayout,
    pipeline: RenderPipeline,
    vertices: Buffer,
    instances_len: usize,
    instances: Buffer,
    indices: Buffer,
    bind_group: BindGroup,
}

impl ModelRenderer {
    pub fn new(rc: &RenderingContext, globals: &Buffer, models: &[Model]) -> Self {
        let shader = rc.device.create_shader_module(&include_wgsl!("shaders/model.wgsl"));

        let bind_group_layout = rc.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("ModelRenderer::bind_group_layout"),
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
            ]
        });

        let pipeline_layout = rc.device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("ModelRenderer::pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = rc.device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("ModelRenderer::pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    Vertex::descriptor(),
                    Instance::descriptor()
                ],
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Cw,
                cull_mode: Some(Face::Back),
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

        let (vertices_len, indices_len) = models.iter().fold((0, 0), |mut acc, model| {
            let (vertices_len, indices_len) = model.meshes.iter().fold((0, 0), |mut acc, mesh| {
                acc.0 += mesh.vertices.len();
                acc.1 += mesh.indices.len();
                acc
            });

            acc.0 += vertices_len;
            acc.1 += indices_len;
            acc
        });

        let vertices = rc.device.create_buffer(&BufferDescriptor {
            label: Some("ModelRenderer::vertices"),
            size: (std::mem::size_of::<Vertex>() * vertices_len) as u64,
            usage: BufferUsages::COPY_DST | BufferUsages::VERTEX,
            mapped_at_creation: false,
        });

        let instances_len = 256;
        let instances = create_instance_buffer(rc, instances_len);

        let indices = rc.device.create_buffer(&BufferDescriptor {
            label: Some("ModelRenderer::indices"),
            size: (std::mem::size_of::<u32>() * indices_len) as u64,
            usage: BufferUsages::COPY_DST | BufferUsages::INDEX,
            mapped_at_creation: false,
        });

        for model in models {
            for mesh in &model.meshes {
                rc.queue.write_buffer(&vertices, 0, bytemuck::cast_slice(&mesh.vertices));
                rc.queue.write_buffer(&indices, 0, bytemuck::cast_slice(&mesh.indices));
            }
        }

        let bind_group = rc.device.create_bind_group(&BindGroupDescriptor {
            label: Some("ModelRenderer::bind_group"),
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: globals.as_entire_binding(),
                },
            ],
        });

        Self {
            shader,
            bind_group_layout,
            pipeline_layout,
            pipeline,
            vertices,
            instances_len,
            instances,
            indices,
            bind_group,
        }
    }

    pub fn render(
        &mut self,
        rc: &RenderingContext,
        surface_view: &TextureView,
        depth_stencil_view: &TextureView,
        instances: &[Instance],
    ) {
        // Rebuild our instance buffer on size mismatch
        if self.instances_len < instances.len() {
            self.instances_len = self.instances_len / 2 * 3;
            self.instances = create_instance_buffer(rc, self.instances_len);
        }

        // Write to our instance buffer
        rc.queue.write_buffer(&self.instances, 0, bytemuck::cast_slice(instances));

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
                        load: LoadOp::Load,
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &depth_stencil_view,
                    depth_ops: Some(Operations {
                        load: LoadOp::Load,
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_vertex_buffer(0, self.vertices.slice(..));
            render_pass.set_vertex_buffer(1, self.instances.slice(..));
            render_pass.set_index_buffer(self.indices.slice(..), IndexFormat::Uint32);
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.draw_indexed(0..864, 0, 0..instances.len() as u32);
        }

        // Submit our work
        rc.queue.submit([command_encoder.finish()]);
    }
}

fn create_instance_buffer(rc: &RenderingContext, instances_len: usize) -> Buffer {
    rc.device.create_buffer(&BufferDescriptor {
        label: Some("ModelRenderer::instances"),
        size: (std::mem::size_of::<Instance>() * instances_len) as u64,
        usage: BufferUsages::COPY_DST | BufferUsages::VERTEX,
        mapped_at_creation: false,
    })
}
