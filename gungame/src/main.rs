mod camera;
mod map;
mod graphics;
mod resolution;

use std::path::PathBuf;
use std::time::Instant;

use anyhow::Result;
use camera::Camera;
use clap::Parser;
use graphics::Renderer;
use log::error;
use map::Map;
use nalgebra::Point3;
use resolution::Resolution;
use winit::dpi::PhysicalSize;
use winit::event::Event;
use winit::event::WindowEvent;
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    map_path: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let args = Args::parse();
    let map_data = match args.map_path {
        Some(p) => std::fs::read_to_string(p)?,
        None => include_str!("cabin.map").to_string(),
    };
    let map = Map::from_str(&map_data)?;
    let mut vertices = vec![];
    let mut vertex_groups = vec![];
    let mut vertex_count = 0;
    for entity in map.entities {
        for brush in entity.brushes {
            let points = brush.vertices();
            vertex_groups.push(vertex_count..vertex_count + points.len() as u32);
            for point in &points {
                vertices.push((*point).into());
            }
            vertex_count += points.len() as u32;
        }
    }

    let event_loop = EventLoop::new();
    let mut resolution = Resolution::new(1280, 720);
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::from(resolution))
        .build(&event_loop)?;

    let mut camera = Camera::default();
    let mut renderer = Renderer::new(&window, resolution).await?;
    renderer.load_map(&vertices, vertex_groups);

    let start_time = Instant::now();
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::Resized(new_size) => {
                        resolution = new_size.into();
                        renderer.resize(new_size.into());
                    },
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        resolution = (*new_inner_size).into();
                        renderer.resize((*new_inner_size).into());
                    },
                    _ => (),
                }
            },
            Event::MainEventsCleared => {
                let elapsed_time = start_time.elapsed().as_secs_f32();
                camera.position = Point3::new(elapsed_time.sin() * 1000.0, 500.0, elapsed_time.cos() * 1000.0);
                camera.look_at(Point3::new(0.0, 0.0, 0.0));
                window.request_redraw();
            },
            Event::RedrawRequested(_) => {
                renderer.write_globals(camera, resolution);
                if let Err(e) = renderer.render() {
                    error!("{e}");
                    *control_flow = ControlFlow::Exit;
                }
            },
            _ => (),
        }
    });
}
