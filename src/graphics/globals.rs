use bytemuck::Pod;
use bytemuck::Zeroable;

use crate::camera::Camera;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Globals {
    proj: [[f32; 4]; 4],
    proj_inv: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
    view_proj: [[f32; 4]; 4],
    cam_pos: [f32; 4],
}

impl Globals {
    pub fn from_camera(camera: &Camera, width: u32, height: u32) -> Self {
        let projection = camera.projection(width, height);
        let view = camera.view().to_homogeneous();

        Self {
            proj: projection.into(),
            proj_inv: projection.try_inverse().unwrap().into(),
            view: view.into(),
            view_proj: (projection * view).into(),
            cam_pos: camera.position.to_homogeneous().into(),
        }
    }
}
