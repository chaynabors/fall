use legion::system;
use nalgebra::Matrix4;
use nalgebra::vector;
use tokio::sync::mpsc::UnboundedSender;

use crate::camera::Camera;
use crate::components::Model;
use crate::components::PlayerBrain;
use crate::components::Position;
use crate::components::Rotation;
use crate::components::Speed;
use crate::components::Velocity;
use crate::graphics::Instance;
use crate::input::InputState;
use crate::time::DeltaTime;

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
