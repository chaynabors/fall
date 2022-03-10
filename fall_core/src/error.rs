use std::fmt::Display;

use nom::error::VerboseError;

pub enum Error<'a> {
    ParserError(VerboseError<&'a str>),
}

impl<'a> Display for Error<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ParserError(e) => write!(f, "{e}"),
        }
    }
}

impl<'a> From<VerboseError<&'a str>> for Error<'a> {
    fn from(from: VerboseError<&'a str>) -> Self {
        Self::ParserError(from)
    }
}
