use bytemuck::Pod;
use bytemuck::Zeroable;
use nalgebra::Matrix3;
use nalgebra::Matrix4;
use wgpu::BufferAddress;
use wgpu::VertexAttribute;
use wgpu::VertexBufferLayout;
use wgpu::VertexFormat;
use wgpu::VertexStepMode;

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Pod, Zeroable)]
pub struct Instance {
    pub model: Matrix4<f32>,
    pub normal: Matrix3<f32>,
}

impl Instance {
    pub fn descriptor<'a>() -> VertexBufferLayout<'a> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as BufferAddress,
            step_mode: VertexStepMode::Instance,
            attributes: &[
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 3,
                },
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: std::mem::size_of::<[f32; 4]>() as BufferAddress,
                    shader_location: 4,
                },
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: std::mem::size_of::<[f32; 8]>() as BufferAddress,
                    shader_location: 5,
                },
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: std::mem::size_of::<[f32; 12]>() as BufferAddress,
                    shader_location: 6,
                },
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: std::mem::size_of::<[f32; 16]>() as BufferAddress,
                    shader_location: 7,
                },
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: std::mem::size_of::<[f32; 19]>() as BufferAddress,
                    shader_location: 8,
                },
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: std::mem::size_of::<[f32; 22]>() as BufferAddress,
                    shader_location: 9,
                },
            ]
        }
    }
}
