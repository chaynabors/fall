use std::f32::consts::PI;

use gilrs::Gilrs;
use nalgebra::Rotation2;
use nalgebra::UnitQuaternion;
use nalgebra::Vector2;
use nalgebra::Vector3;
use winit::event::ElementState;
use winit::event::KeyboardInput;

use crate::error::Error;

const W: u32 = 0x11;
const A: u32 = 0x1E;
const S: u32 = 0x1F;
const D: u32 = 0x20;

const MAGIC_DELTA_MULTIPLIER: f32 = 0.005;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InputState {
    pub move_direction: Vector2<f32>,
    pub view_direction: UnitQuaternion<f32>,
}

#[derive(Debug)]
pub struct Input {
    gilrs: Gilrs,
    move_forward: u8,
    move_backward: u8,
    strafe_left: u8,
    strafe_right: u8,
    move_analog: Vector2<f32>,
    view_pitch: f32,
    view_yaw: f32,
}

impl Input {
    pub fn new() -> Result<Self, Error> {
        let gilrs = Gilrs::new()?;

        Ok(Self {
            gilrs,
            move_forward: 0,
            move_backward: 0,
            strafe_left: 0,
            strafe_right: 0,
            move_analog: Vector2::zeros(),
            view_pitch: 0.0,
            view_yaw: 0.0,
        })
    }

    pub fn update_key_state(&mut self, event: KeyboardInput) {
        let state = match event.state {
            ElementState::Pressed => 1,
            ElementState::Released => 0,
        };

        match event.scancode {
            i if i == W => self.move_forward = state,
            i if i == S => self.move_backward = state,
            i if i == A => self.strafe_left = state,
            i if i == D => self.strafe_right = state,
            _ => (),
        }
    }

    pub fn apply_mouse_delta(&mut self, delta: (f64, f64)) {
        self.view_pitch += delta.1 as f32 * MAGIC_DELTA_MULTIPLIER;
        self.view_pitch = self.view_pitch.clamp(-PI * 0.4, PI * 0.4);
        self.view_yaw += delta.0 as f32 * MAGIC_DELTA_MULTIPLIER;
        self.view_yaw %= 2.0 * PI;
    }

    pub fn get_state(&mut self) -> InputState {
        while let Some(_event) = self.gilrs.next_event() {
            // TODO: handle the event
        }

        let mut move_direction = self.move_analog;
        move_direction.y += self.move_forward as u32 as f32;
        move_direction.y -= self.move_backward as u32 as f32;
        move_direction.x -= self.strafe_left as u32 as f32;
        move_direction.x += self.strafe_right as u32 as f32;
        move_direction = Rotation2::new(-self.view_yaw) * move_direction;
        move_direction = move_direction.cap_magnitude(1.0);
        
        let mut view_direction = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), self.view_yaw);
        view_direction = view_direction * UnitQuaternion::from_axis_angle(&Vector3::x_axis(), self.view_pitch);

        InputState {
            move_direction,
            view_direction,
        }
    }
}
