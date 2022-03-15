#![allow(dead_code)]

use nalgebra::Isometry3;
use nalgebra::Matrix4;
use nalgebra::Point3;
use nalgebra::UnitQuaternion;
use nalgebra::Vector3;
use nalgebra_glm::reversed_infinite_perspective_rh_zo;

#[derive(Clone, Copy, Debug)]
pub struct Camera {
    pub position: Point3<f32>,
    pub rotation: UnitQuaternion<f32>,
    pub fov: f32,
}

impl Camera {
    /// fov is in degrees
    pub fn new(position: Point3<f32>, rotation: UnitQuaternion<f32>, fov: f32) -> Self {
        Self { position, rotation, fov: fov.to_radians() }
    }

    pub fn view(&self) -> Isometry3<f32> {
        Isometry3::look_at_rh(
            &self.position,
            &(self.position - (self.rotation.inverse() * Vector3::z())),
            &Vector3::y_axis(),
        )
    }

    pub fn view_origin(&self) -> Isometry3<f32> {
        Isometry3::look_at_rh(
            &Point3::origin(),
            &(Point3::origin() - (self.rotation.inverse() * Vector3::z())),
            &Vector3::y_axis(),
        )
    }

    pub fn projection(&self, width: u32, height: u32) -> Matrix4<f32> {
        reversed_infinite_perspective_rh_zo(width as f32 / height as f32, self.fov, 0.1)
    }

    pub fn look_at(&mut self, target: Point3<f32>) {
        self.rotation = UnitQuaternion::look_at_rh(&(target - self.position), &Vector3::y_axis());
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Point3::origin(),
            rotation: UnitQuaternion::identity(),
            fov: 90_f32.to_radians(),
        }
    }
}
