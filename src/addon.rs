use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

use crate::error::Error;

#[derive(Debug, Deserialize, Serialize)]
pub struct Addon {
    /// A descriptive name seen in game
    pub name: String,
    /// A description seen in game
    pub description: String,
    /// All of the dependencies this addon has
    pub dependencies: Vec<String>,
    /// The relative paths the the maps this addon adds
    pub maps: Vec<PathBuf>,
    /// The relative paths to the model files this addon requires
    ///
    /// Supported file types are:
    /// - `obj`: An open file format without support for animation
    pub models: Vec<PathBuf>,
}

impl Addon {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let file = std::fs::File::open(path)?;
        let reader = BufReader::new(file);
        let addon = serde_json::from_reader(reader)?;
        Ok(addon)
    }
}
