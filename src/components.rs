use nalgebra::Point3;
use nalgebra::UnitQuaternion;
use nalgebra::Vector3;
use winit::dpi::PhysicalSize;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Model(pub u32);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlayerBrain;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position(pub Point3<f32>);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

impl From<PhysicalSize<u32>> for Resolution {
    fn from(from: PhysicalSize<u32>) -> Self {
        Self { width: from.width, height: from.height }
    }
}

impl Into<PhysicalSize<u32>> for Resolution {
    fn into(self) -> PhysicalSize<u32> {
        PhysicalSize { width: self.width, height: self.height }
    }
}

impl From<(u32, u32)> for Resolution {
    fn from(from: (u32, u32)) -> Self {
        Self { width: from.0, height: from.1 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rotation(pub UnitQuaternion<f32>);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Speed(pub f32);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Velocity(pub Vector3<f32>);
