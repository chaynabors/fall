mod addon;
mod camera;
mod components;
mod error;
mod graphics;
mod input;
mod time;
mod systems;

use std::collections::HashMap;
use std::net::Ipv4Addr;

use clap::Parser;
use legion::Resources;
use legion::Schedule;
use legion::World;
use mappy::Map;
use nalgebra::Point3;
use nalgebra::UnitQuaternion;
use nalgebra::Vector3;
use tokio::sync::mpsc;
use tracing::error;
use tracing::info;
use tracing::warn;
use winit::dpi::PhysicalSize;
use winit::event::DeviceEvent;
use winit::event::Event;
use winit::event::WindowEvent;
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

use self::addon::Addon;
use self::camera::Camera;
use self::components::PlayerBrain;
use self::components::Position;
use self::components::Resolution;
use self::components::Rotation;
use self::components::Speed;
use self::components::Velocity;
use self::error::Error;
use self::graphics::Graphics;
use self::input::Input;
use self::systems::render_models_system;
use self::systems::update_player_camera_system;
use self::systems::update_positions_system;
use self::systems::update_player_velocities_system;
use self::time::Time;

const GAME_NAME: &str = env!("CARGO_PKG_NAME");
const GAME_NAME_DISPLAY: &str = "Gungame";

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Name of the addon to load, defaults to the base game
    addon: Option<String>,
    /// Address of the server host (if any)
    host: Option<Ipv4Addr>,
}

type Result<T> = std::result::Result<T, Error>;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    // Acquire all of our useful directories
    let userdirs = directories::UserDirs::new().ok_or(Error::NoUserDirectory)?;
    let document_dir = userdirs.document_dir().ok_or(Error::NoDocumentDirectory)?;
    let game_dir = document_dir.join(GAME_NAME_DISPLAY);
    let addons_dir = game_dir.join("addons");

    // Load the given addon or base game otherwise
    let addon_name = args.addon.unwrap_or(GAME_NAME.to_string());
    let addon_dir = addons_dir.join(&addon_name);
    let addon = Addon::from_path(&addon_dir.join(format!("{addon_name}.json")))?;

    // Load our map
    let map_path = match addon.maps.first() {
        Some(map) => addon_dir.join(map.1),
        None => {
            warn!("No maps to load, exiting...");
            return Ok(());
        },
    };
    let map_data = std::fs::read_to_string(map_path)?;
    let map = Map::from_str(&map_data)?;

    // Load models
    let mut model_indices = HashMap::new();
    let mut models = vec![];
    for (i, (name, path)) in addon.models.iter().enumerate() {
        model_indices.insert(name, i);
        models.push(graphics::Model::from_obj(&addon_dir.join(path))?);
    }

    // Load textures
    let mut texture_indices = HashMap::new();
    let mut textures = vec![];
    for (i, (name, path)) in addon.textures.iter().enumerate() {
        texture_indices.insert(name, i);
        textures.push(graphics::Texture::from_file(&addon_dir.join(path))?);
    }

    // Set up our event loop
    let event_loop = EventLoop::new();
    let mut resolution = Resolution { width: 1280, height: 720 };
    let window = WindowBuilder::new()
        .with_inner_size::<PhysicalSize<u32>>(resolution.into())
        .build(&event_loop)?;
    let mut scale_factor = window.scale_factor();

    let (instance_sender, instance_receiver) = mpsc::unbounded_channel();

    let mut graphics = Graphics::new(&window, resolution, &map, &models, instance_receiver).await?;
    let mut input = Input::new()?;
    let mut time = Time::new();

    let mut world = World::default();

    let player = world.push((
        components::Model(1),
        PlayerBrain,
        Position(Point3::origin()),
        Rotation(UnitQuaternion::identity()),
        Velocity(Vector3::zeros()),
        Speed(5.64),
    ));

    let mut resources = Resources::default();
    resources.insert(Camera::default());
    resources.insert(instance_sender);

    let mut logic_scheduler = Schedule::builder()
        .add_system(update_player_velocities_system())
        .add_system(update_positions_system())
        .add_system(render_models_system())
        .add_system(update_player_camera_system())
        .build();

    let mut frame_count = 0;
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(new_size) => resolution = new_size.into(),
                WindowEvent::CloseRequested => {
                    info!("average fps: {}", frame_count / time.elapsed_time().0 as u32);
                    *control_flow = ControlFlow::Exit;
                },
                WindowEvent::ScaleFactorChanged { scale_factor: sf, new_inner_size } => {
                    scale_factor = sf;
                    resolution = (*new_inner_size).into();
                },
                _ => (),
            },
            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::MouseMotion { delta } => input.apply_mouse_delta(delta),
                DeviceEvent::Key(key_state) => input.update_key_state(key_state),
                _ => (),
            }
            Event::MainEventsCleared => {
                let input = input.get_state();
                let mut camera: Camera = *resources.get_mut().unwrap();
                camera.rotation = input.view_direction;

                resources.insert(time.elapsed_time());
                resources.insert(time.delta_time());
                resources.insert(input);
                logic_scheduler.execute(&mut world, &mut resources);

                if let Err(e) = graphics.render(resolution, &camera) {
                    error!("{e}");
                    *control_flow = ControlFlow::Exit;
                }

                frame_count += 1;
            },
            _ => (),
        }
    });
}
