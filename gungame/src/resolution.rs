use winit::dpi::PhysicalSize;

#[derive(Clone, Copy, Debug)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

impl Resolution {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
        }
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }
}

impl Default for Resolution {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
        }
    }
}

impl From<PhysicalSize<u32>> for Resolution {
    fn from(from: PhysicalSize<u32>) -> Self {
        Self {
            width: from.width,
            height: from.height,
        }
    }
}

impl From<Resolution> for PhysicalSize<u32> {
    fn from(from: Resolution) -> Self {
        PhysicalSize {
            width: from.width,
            height: from.height
        }
    }
}

impl From<Resolution> for [u32; 2] {
    fn from(from: Resolution) -> Self {
        [from.width, from.height]
    }
}

impl From<Resolution> for [f32; 2] {
    fn from(from: Resolution) -> Self {
        [from.width as f32, from.height as f32]
    }
}

impl From<Resolution> for (i32, i32) {
    fn from(from: Resolution) -> Self {
        (from.width as i32, from.height as i32)
    }
}
