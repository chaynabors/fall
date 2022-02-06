mod error;
mod map;

use std::path::PathBuf;

use clap::Parser;
use error::Error;
use map::Map;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    map: PathBuf,
}

fn main() {
    let args = Args::parse();
    let map = Map::from_file(args.map).map_err(|e| format!("failed to parse map: {e}"));
    println!("{map:#?}");
    //let dir = args.map
}
