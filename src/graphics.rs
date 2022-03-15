use mappy::Map;
use rendering_util::RenderingContext;
use winit::window::Window;

use crate::camera::Camera;
use crate::ecs::Resolution;
use crate::error::Error;
use crate::map_renderer::MapRenderer;

pub struct Graphics {
    rendering_context: RenderingContext,
    map_renderer: MapRenderer,
}

impl Graphics {
    pub async fn new(window: &Window, resolution: Resolution, map: &Map<'_>) -> Result<Self, Error> {
        let width = resolution.width;
        let height = resolution.height;
        let rendering_context = RenderingContext::new(&window, width, height).await?;
        let mut map_renderer = MapRenderer::new(&rendering_context, &map);
        map_renderer.load_map(&rendering_context, &map);

        Ok(Self {
            rendering_context,
            map_renderer,
        })
    }

    pub fn render(&mut self, resolution: Resolution, camera: &Camera) -> Result<(), Error> {
        let width = resolution.width;
        let height = resolution.height;
        self.rendering_context.render(width, height, |rc, view| {
            self.map_renderer.render(rc, view, &camera);
        })?;

        Ok(())
    }
}
