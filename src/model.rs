#![allow(dead_code)]

use std::fmt::Debug;
use std::path::Path;

use bytemuck::Pod;
use bytemuck::Zeroable;
use nalgebra::point;
use nalgebra::Point2;
use nalgebra::Point3;
use nalgebra::UnitVector3;
use nalgebra::vector;
use wgpu::BufferAddress;
use wgpu::VertexAttribute;
use wgpu::VertexBufferLayout;
use wgpu::VertexFormat;
use wgpu::VertexStepMode;

use crate::error::Error;

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Pod, Zeroable)]
pub struct Vertex {
    pub position: Point3<f32>,
    pub normal: UnitVector3<f32>,
    pub tex_coord: Point2<f32>,
}

impl Vertex {
    pub fn descriptor<'a>() -> VertexBufferLayout<'a> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    format: VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                VertexAttribute {
                    format: VertexFormat::Float32x3,
                    offset: std::mem::size_of::<[f32; 3]>() as BufferAddress,
                    shader_location: 1,
                },
                VertexAttribute {
                    format: VertexFormat::Float32x3,
                    offset: std::mem::size_of::<[f32; 6]>() as BufferAddress,
                    shader_location: 2,
                },
            ],
        }
    }
}

#[derive(Clone, Debug)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

#[derive(Clone, Debug)]
pub struct Model {
    pub meshes: Vec<Mesh>,
}

impl Model {
    pub fn from_obj<P: AsRef<Path> + Debug>(path: P) -> Result<Self, Error> {
        let (models, _materials) = tobj::load_obj(
            &path,
            &tobj::LoadOptions {
                single_index: true,
                triangulate: true,
                ignore_points: true,
                ignore_lines: true,
            }
        )?;

        let mut meshes = vec![];
        for model in models {
            if model.mesh.normals.len() == 0 { return Err(Error::MeshWithoutNormals); }
            if model.mesh.texcoords.len() == 0 { return Err(Error::MeshWithoutTexCoords); }

            let mut vertices = vec![];
            for i in 0..model.mesh.positions.len() / 3 {
                vertices.push(Vertex {
                    position: point![
                        model.mesh.positions[i * 3],
                        model.mesh.positions[i * 3 + 1],
                        model.mesh.positions[i * 3 + 2]
                    ],
                    normal: UnitVector3::new_unchecked(vector![
                        model.mesh.normals[i * 3],
                        model.mesh.normals[i * 3 + 1],
                        model.mesh.normals[i * 3 + 2]
                    ]),
                    tex_coord: point![
                        model.mesh.texcoords[i * 2],
                        model.mesh.texcoords[i * 2 + 1]
                    ],
                });
            }

            meshes.push(Mesh {
                vertices,
                indices: model.mesh.indices,
            });
        }

        Ok(Self {
            meshes,
        })
    }
}
