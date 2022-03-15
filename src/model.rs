#![allow(dead_code)]

use std::fmt::Debug;
use std::path::Path;

use nalgebra::point;
use nalgebra::Point2;
use nalgebra::Point3;
use nalgebra::UnitVector3;
use nalgebra::vector;

use crate::error::Error;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vertex {
    pub position: Point3<f32>,
    pub normal: UnitVector3<f32>,
    pub tex_coord: Point2<f32>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

#[derive(Clone, Debug, PartialEq)]
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
