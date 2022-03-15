use legion::system;
use nalgebra::Point3;
use nalgebra::Vector3;
use winit::dpi::PhysicalSize;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position(pub Point3<f32>);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Resolution {
    ///
    pub width: u32,
    ///
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Model(u32);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Speed(pub f32);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Velocity(pub Vector3<f32>);

#[system(for_each)]
pub fn update_positions(
    position: &mut Position,
    velocity: &Velocity,
    #[resource] delta_time: &f32,
) {
    position.0 += velocity.0 * *delta_time;
    println!("{:?}", position.0);
}

#[system(for_each)]
pub fn update_velocities(
    velocity: &mut Velocity,
    speed: &Speed,
    #[resource] dir: &Option<Vector3<f32>>,
) {
    match dir {
        Some(dir) => velocity.0 = dir.scale(speed.0),
        None => velocity.0 = Vector3::zeros(),
    }
}
