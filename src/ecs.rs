use legion::system;
use nalgebra::Matrix4;
use nalgebra::Point3;
use nalgebra::UnitQuaternion;
use nalgebra::vector;
use nalgebra::Vector3;
use tokio::sync::mpsc::UnboundedSender;
use winit::dpi::PhysicalSize;

use crate::camera::Camera;
use crate::input::InputState;
use crate::model_renderer::Instance;
use crate::time::DeltaTime;

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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rotation(pub UnitQuaternion<f32>);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Speed(pub f32);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Velocity(pub Vector3<f32>);

#[system(for_each)]
pub fn update_positions(
    #[resource] delta_time: &DeltaTime,
    position: &mut Position,
    velocity: &Velocity,
) {
    position.0 += velocity.0.scale(delta_time.0);
}

#[system(for_each)]
pub fn update_player_velocities(
    #[resource] input: &InputState,
    velocity: &mut Velocity,
    player_brain: &PlayerBrain,
    speed: &Speed,
) {
    let dir = input.move_direction;
    velocity.0 = vector![dir.x, 0.0, dir.y].scale(speed.0);
}

#[system(for_each)]
pub fn update_player_camera(
    #[resource] camera: &mut Camera,
    _player_brain: &PlayerBrain,
    position: &Position,
    rotation: &Rotation,
) {
    let mut position = position.0;
    position.y += 1.65;
    camera.position = position;
    camera.rotation = rotation.0;
}

#[system(for_each)]
pub fn render_models(
    #[resource] send: &UnboundedSender<Instance>,
    _model: &Model,
    position: &Position,
    rotation: &Rotation,
) {
    // TODO: unwrapping is a code smell
    send.send(Instance {
        model: Matrix4::new_translation(&position.0.coords),
        normal: rotation.0.into(),
    }).unwrap();
}
