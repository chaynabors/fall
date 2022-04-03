use std::collections::HashMap;
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;

use indexmap::IndexMap;
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
    /// The names of the addons this addon depends on
    pub dependencies: Vec<String>,
    /// A collection of maps by their internal name, the first map has precedence
    pub maps: IndexMap<String, PathBuf>,
    /// A collection of models by their internal name
    ///
    /// Supported file types are:
    /// - `obj`: An open file format without support for animation
    pub models: HashMap<String, PathBuf>,
    /// A collection of textures by their internal name
    ///
    /// Supported file types are:
    /// - `png` Recommended
    /// - `jpg`
    /// - `gif`
    /// - `bmp`
    pub textures: HashMap<String, PathBuf>,
}

impl Addon {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let file = std::fs::File::open(path)?;
        let reader = BufReader::new(file);
        let addon = serde_json::from_reader(reader)?;
        Ok(addon)
    }
}
