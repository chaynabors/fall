use std::fmt::Debug;
use std::path::Path;

use image::GenericImageView;

use crate::Result;
use crate::components::Resolution;

pub struct Texture {
    pub data: Vec<u8>,
    pub resolution: Resolution,
}

impl Texture {
    pub fn from_file<P: AsRef<Path> + Debug>(path: P) -> Result<Self> {
        let image = image::open(path)?;
        let resolution = image.dimensions().into();

        Ok(Self {
            data: image.into_rgb8().into_vec(),
            resolution,
        })
    }
}
