use nalgebra::Vector3;

#[derive(Clone, Debug, Default)]
pub struct Plane {
    pub points: [Vector3<f32>; 3],
    pub texture: String,
    pub x_offset: f32,
    pub y_offset: f32,
    pub rotation: f32,
    pub x_scale: f32,
    pub y_scale: f32,
}
