use std::collections::HashMap;
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
    /// The version number used for syncing online play
    pub version: u32,
    /// Collection of all the authors of the addon
    pub authors: Vec<String>,
    /// All of the dependencies this addon has
    pub dependencies: Vec<String>,
    /// A collection of maps by their internal name
    pub maps: HashMap<String, PathBuf>,
    /// A collection of models by their internal name
    ///
    /// Supported file types are:
    /// - `obj`: An open file format without support for animation
    pub models: HashMap<String, PathBuf>,
}

impl Addon {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let file = std::fs::File::open(path)?;
        let reader = BufReader::new(file);
        let addon = serde_json::from_reader(reader)?;
        Ok(addon)
    }
}
