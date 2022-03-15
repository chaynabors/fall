mod addon;
mod camera;
mod ecs;
mod error;
mod graphics;
mod map_renderer;
mod model;

use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::time::Instant;

use addon::Addon;
use camera::Camera;
use clap::Parser;
use ecs::Position;
use ecs::Speed;
use ecs::update_positions_system;
use ecs::update_velocities_system;
use ecs::Velocity;
use error::Error;
use legion::Resources;
use legion::Schedule;
use legion::World;
use log::error;
use log::warn;
use mappy::Map;
use nalgebra::Point3;
use nalgebra::vector;
use nalgebra::Vector3;
use winit::dpi::PhysicalSize;
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
    env_logger::init();

    let args = Args::parse();

    // Load our addon
    let addon_path: PathBuf = "addons/fall/fall.json".into();
    let addon = Addon::from_path(&addon_path)?;
    if addon.maps.len() == 0 {
        warn!("No maps to load, exiting...");
        return Ok(());
    }

    // Load our map
    let map_data = match args.map {
        Some(path) => std::fs::read_to_string(path)?,
        None => std::fs::read_to_string(
            addon_path.parent().unwrap().join(&addon.maps[0])
        )?
    };
    let map = Map::from_str(&map_data)?;

    let event_loop = EventLoop::new();
    let mut resolution = Resolution { width: 1280, height: 720 };
    let window = WindowBuilder::new()
        .with_inner_size::<PhysicalSize<u32>>(resolution.into())
        .build(&event_loop)?;

    let mut graphics = Graphics::new(&window, resolution, &map).await?;

    let mut world = World::default();

    let player = world.push((
        Position(Point3::origin()),
        Velocity(Vector3::zeros()),
        Speed(100.0),
    ));

    let mut resources = Resources::default();
    resources.insert(Camera::default());
    resources.insert(0.0 as f32); // delta_time
    resources.insert(None::<Vector3<f32>>);

    let mut logic_scheduler = Schedule::builder()
        .add_system(update_velocities_system())
        .add_system(update_positions_system())
        .build();

    let start_time = Instant::now();
    let mut frame_time = Instant::now();
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::Resized(new_size) => resolution = new_size.into(),
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        resolution = (*new_inner_size).into()
                    },
                    _ => (),
                }
            },
            Event::MainEventsCleared => {
                let elapsed_time = start_time.elapsed().as_secs_f32() * 0.5;
                let delta_time = frame_time.elapsed().as_secs_f32();
                frame_time = Instant::now();

                let mut camera: Camera = *resources.get_mut().unwrap();
                camera.position = Point3::new(elapsed_time.sin() * 1000.0, 500.0, elapsed_time.cos() * 1000.0);
                camera.look_at(Point3::new(0.0, 0.0, 0.0));

                resources.insert(delta_time);
                resources.insert(Some(vector![0.0 as f32, 0.0, 1.0]));
                logic_scheduler.execute(&mut world, &mut resources);

                if let Err(e) = graphics.render(resolution, &camera) {
                    error!("{e}");
                    *control_flow = ControlFlow::Exit;
                }
            },
            _ => (),
        }
    });
}
