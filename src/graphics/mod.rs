mod globals;
mod graphics;
mod instance;
mod map_renderer;
mod model;
mod model_renderer;
mod texture;

pub use self::graphics::Graphics;
pub use self::instance::Instance;
pub use self::model::Model;
pub use self::texture::Texture;

use wgpu::TextureFormat;

use self::globals::Globals;
use self::map_renderer::MapRenderer;
use self::model::Vertex;
use self::model_renderer::ModelRenderer;

pub const DEPTH_FORMAT: TextureFormat = TextureFormat::Depth32Float;
