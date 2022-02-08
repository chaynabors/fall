mod convex_hull;
mod map;

use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use clap::Parser;
use map::Map;
use nalgebra::Vector3;

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    map_path: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let map = Map::load(&args.map_path)
        .with_context(|| format!("failed to parse map at: {}", args.map_path.display()))?;

    let mut half_planes = vec![];
    for entity in map.entities {
        for brush in entity.brushes {
            for plane in brush {
                let points = plane.points;
                let normal = Vector3::cross(&(points[2] - points[0]), &(points[1] - points[0])).normalize();
                half_planes.push((points[0], normal));
            }
        }
    }

    println!("{half_planes:?}");

    Ok(())
}
