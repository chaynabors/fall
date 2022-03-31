use bytemuck::Pod;
use bytemuck::Zeroable;
use mappy::Map;
use rendering_util::RenderingContext;
use tokio::sync::mpsc::UnboundedReceiver;
use wgpu::Buffer;
use wgpu::BufferUsages;
use wgpu::Device;
use wgpu::Extent3d;
use wgpu::Texture;
use wgpu::TextureDescriptor;
use wgpu::TextureDimension;
use wgpu::TextureFormat;
use wgpu::TextureUsages;
use wgpu::TextureView;
use wgpu::TextureViewDescriptor;
use wgpu::util::BufferInitDescriptor;
use wgpu::util::DeviceExt;
use winit::window::Window;

use crate::camera::Camera;
use crate::ecs::Resolution;
use crate::error::Error;
use crate::map_renderer::MapRenderer;
use crate::model::Model;
use crate::model_renderer::Instance;
use crate::model_renderer::ModelRenderer;

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
    pub fn from_camera(camera: &Camera, width: u32, height: u32) -> Self {
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

pub struct Graphics {
    rendering_context: RenderingContext,
    instance_receiver: UnboundedReceiver<Instance>,
    depth_stencil: Texture,
    depth_stencil_view: TextureView,
    globals: Buffer,
    map_renderer: MapRenderer,
    model_renderer: ModelRenderer,
}

impl Graphics {
    pub async fn new(
        window: &Window,
        resolution: Resolution,
        map: &Map<'_>,
        models: &[Model],
        instance_receiver: UnboundedReceiver<Instance>,
    ) -> Result<Self, Error> {
        let width = resolution.width;
        let height = resolution.height;

        let rc = RenderingContext::new(&window, width, height).await?;

        let (depth_stencil, depth_stencil_view) = create_depth_stencil(&rc.device, width, height);

        let globals = rc.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("globals"),
            contents: bytemuck::bytes_of(
                &Globals::from_camera(&Camera::default(), width, height)
            ),
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
        });

        let map_renderer = MapRenderer::new(&rc, &globals, &map);
        let model_renderer = ModelRenderer::new(&rc, &globals, models);

        Ok(Self {
            rendering_context: rc,
            instance_receiver,
            depth_stencil,
            depth_stencil_view,
            globals,
            map_renderer,
            model_renderer,
        })
    }

    pub fn render(&mut self, resolution: Resolution, camera: &Camera) -> Result<(), Error> {
        let width = resolution.width;
        let height = resolution.height;

        // Resize our frame
        let rc = &self.rendering_context;
        if rc.width() != width || rc.height() != height {
            let (ds, dsv) = create_depth_stencil(&rc.device, width, height);
            self.depth_stencil = ds;
            self.depth_stencil_view = dsv;
        }

        // Write our globals
        rc.queue.write_buffer(
            &self.globals,
            0,
            bytemuck::bytes_of(&Globals::from_camera(camera, width, height),
        ));

        // Do our rendering
        self.rendering_context.render(width, height, |rc, surface_view| {
            self.map_renderer.render(rc, surface_view, &self.depth_stencil_view);

            let mut instances = vec![];
            while let Ok(instance) = self.instance_receiver.try_recv() {
                instances.push(instance);
            }

            self.model_renderer.render(rc, surface_view, &self.depth_stencil_view, &instances);
        })?;

        Ok(())
    }
}

// Helper function for building our depth_stencil
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
