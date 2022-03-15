use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    IOError(std::io::Error),
    JsonError(serde_json::Error),
    MapError(mappy::Error),
    ObjError(tobj::LoadError),
    MeshWithoutNormals,
    MeshWithoutTexCoords,
    WinitError(winit::error::OsError),
    RenderUtilError(rendering_util::Error),
}

impl<'a> Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IOError(e) => e.fmt(f),
            Error::JsonError(e) => e.fmt(f),
            Error::MapError(e) => e.fmt(f),
            Error::ObjError(e) => e.fmt(f),
            Error::MeshWithoutNormals => write!(f, "Attempted to load a mesh without normals"),
            Error::MeshWithoutTexCoords => write!(f, "Attempted to load a mesh without tex_coords"),
            Error::WinitError(e) => e.fmt(f),
            Error::RenderUtilError(e) => e.fmt(f),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(from: std::io::Error) -> Self {
        Self::IOError(from)
    }
}

impl From<serde_json::Error> for Error {
    fn from(from: serde_json::Error) -> Self {
        Self::JsonError(from)
    }
}

impl From<mappy::Error> for Error {
    fn from(from: mappy::Error) -> Self {
        Self::MapError(from)
    }
}

impl From<tobj::LoadError> for Error {
    fn from(from: tobj::LoadError) -> Self {
        Self::ObjError(from)
    }
}

impl From<winit::error::OsError> for Error {
    fn from(from: winit::error::OsError) -> Self {
        Self::WinitError(from)
    }
}

impl From<rendering_util::Error> for Error {
    fn from(from: rendering_util::Error) -> Self {
        Self::RenderUtilError(from)
    }
}
