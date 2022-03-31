mod addon;
mod camera;
mod ecs;
mod error;
mod graphics;
mod input;
mod map_renderer;
mod model;
mod model_renderer;
mod time;

use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::path::PathBuf;

use addon::Addon;
use camera::Camera;
use clap::Parser;
use ecs::PlayerBrain;
use ecs::Position;
use ecs::Rotation;
use ecs::Speed;
use ecs::render_models_system;
use ecs::update_player_camera_system;
use ecs::update_positions_system;
use ecs::update_player_velocities_system;
use ecs::Velocity;
use error::Error;
use input::Input;
use legion::Resources;
use legion::Schedule;
use legion::World;
use log::LevelFilter;
use log::error;
use log::info;
use log::warn;
use mappy::Map;
use model::Model;
use nalgebra::Point3;
use nalgebra::UnitQuaternion;
use nalgebra::Vector3;
use time::Time;
use tokio::sync::mpsc;
use winit::dpi::PhysicalSize;
use winit::event::DeviceEvent;
use winit::event::Event;
use winit::event::WindowEvent;
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

use crate::ecs::Resolution;
use crate::graphics::Graphics;

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    map: Option<PathBuf>,
    host: Option<Ipv4Addr>,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::builder().filter_level(LevelFilter::Info).init();
    let args = Args::parse();

    // Load our addon
    let addon_path: PathBuf = "addons/fall".into();
    let addon_name = "fall.json";
    let addon = Addon::from_path(&addon_path.join(&addon_name))?;
    if addon.maps.len() == 0 {
        warn!("No maps to load, exiting...");
        return Ok(());
    }

    // Load our map
    let map_data = match args.map {
        Some(path) => std::fs::read_to_string(path)?,
        None => std::fs::read_to_string(
            addon_path.join(&addon.maps["cabin"])
        )?
    };
    let map = Map::from_str(&map_data)?;

    let mut model_indices_by_name = HashMap::new();
    let mut models = vec![];
    for (i, (name, path)) in addon.models.iter().enumerate() {
        model_indices_by_name.insert(name, i);
        models.push(Model::from_obj(&addon_path.join(path))?);
    }

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
        ecs::Model(1),
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
                    resolution = (*new_inner_size).into()
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
