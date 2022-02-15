use std::hash::Hash;

use bytemuck::Pod;
use bytemuck::Zeroable;
use nalgebra::Vector3;
use wgpu::VertexAttribute;
use wgpu::VertexFormat;

pub const VERTEX_ATTRIBUTES: &[VertexAttribute] = &[
    VertexAttribute {
        format: VertexFormat::Float32x3,
        offset: 0,
        shader_location: 0,
    },
];

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
}

impl Vertex {
    pub fn new(position: [f32; 3]) -> Self {
        Self { position }
    }
}

impl Eq for Vertex {}

impl Hash for Vertex {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.position[0] as u32).hash(state);
        (self.position[1] as u32).hash(state);
        (self.position[2] as u32).hash(state);
    }
}

impl PartialEq for Vertex {
    fn eq(&self, other: &Self) -> bool {
        if (self.position[0] - other.position[0]).abs() > 0.1 { return false; }
        if (self.position[1] - other.position[1]).abs() > 0.1 { return false; }
        if (self.position[2] - other.position[2]).abs() > 0.1 { return false; }
        true
    }
}

impl From<Vector3<f32>> for Vertex {
    fn from(from: Vector3<f32>) -> Self {
        Self::new(from.into())
    }
}
