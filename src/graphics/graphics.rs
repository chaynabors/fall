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
use crate::components::Resolution;
use crate::error::Error;

use super::DEPTH_FORMAT;
use super::Globals;
use super::Instance;
use super::MapRenderer;
use super::Model;
use super::ModelRenderer;

pub struct Graphics {
    rendering_context: RenderingContext,
    instance_receiver: UnboundedReceiver<Instance>,
    depth_stencil: Texture,
    depth_stencil_view: TextureView,
    globals: Buffer,
    map_renderer: MapRenderer,
    model_renderer: ModelRenderer,
    textures: Vec<Texture>,
    texture_views: Vec<TextureView>,
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
            textures: vec![],
            texture_views: vec![],
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

    pub fn load_texture(&mut self, texture: &super::Texture) {
        let rc = &self.rendering_context;
        let texture = rc.device.create_texture(&TextureDescriptor {
            label: None,
            size: Extent3d {
                width: texture.resolution.width,
                height: texture.resolution.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING,
        });

        let view = texture.create_view(&TextureViewDescriptor::default());

        self.textures.push(texture);
        self.texture_views.push(view);
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
