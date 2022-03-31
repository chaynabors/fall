#![allow(dead_code)]

use nalgebra::Isometry3;
use nalgebra::Matrix4;
use nalgebra::Point3;
use nalgebra::UnitQuaternion;
use nalgebra::Vector3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera {
    pub position: Point3<f32>,
    pub rotation: UnitQuaternion<f32>,
    pub fov: f32,
    pub near: f32,
}

impl Camera {
    /// fov is in degrees
    pub fn new(position: Point3<f32>, rotation: UnitQuaternion<f32>, fov: f32) -> Self {
        Self { position, rotation, fov: fov.to_radians(), near: 0.01 }
    }

    pub fn view(&self) -> Isometry3<f32> {
        Isometry3::look_at_lh(
            &self.position,
            &(self.position.coords + (self.rotation * Vector3::z())).into(),
            &Vector3::y_axis(),
        )
    }

    pub fn view_origin(&self) -> Isometry3<f32> {
        Isometry3::look_at_lh(
            &Point3::origin(),
            &(self.rotation * Vector3::z()).into(),
            &Vector3::y_axis(),
        )
    }

    pub fn projection(&self, width: u32, height: u32) -> Matrix4<f32> {
        let sy = 1.0 / f32::tan(self.fov * 0.5);
        let sx = sy * (height as f32 / width as f32);

        Matrix4::new(
            sx, 0.0, 0.0, 0.0,
            0.0, sy, 0.0, 0.0,
            0.0, 0.0, 0.0, self.near,
            0.0, 0.0, 1.0, 0.0
        )
    }

    pub fn look_at(&mut self, target: Point3<f32>) {
        self.rotation = UnitQuaternion::look_at_lh(&(target - self.position), &Vector3::y_axis());
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Point3::origin(),
            rotation: UnitQuaternion::identity(),
            fov: 90_f32.to_radians(),
            near: 0.01,
        }
    }
}
