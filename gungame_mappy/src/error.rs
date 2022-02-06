use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    ParseError(String),
    IncompleteData(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IoError(e) => e.fmt(f),
            Error::ParseError(e) => e.fmt(f),
            Error::IncompleteData(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(from: std::io::Error) -> Self {
        Self::IoError(from)
    }
}
