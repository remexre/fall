use std::fmt::{Display, Formatter, Result as FmtResult};

use cst::ParseError;

/// An error loading a file.
#[derive(Debug, Fail)]
pub enum LoadError {
    /// An error reading the file.
    Io(::std::io::Error),

    /// An error parsing the file.
    Parse(ParseError),
}

impl Display for LoadError {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        match *self {
            LoadError::Io(ref err) => write!(fmt, "{}", err),
            LoadError::Parse(ref err) => write!(fmt, "{}", err.clone().map_token(|(_n, t)| t)),
        }
    }
}

/// An error during resolution.
#[derive(Debug, Fail)]
pub enum ResolutionError {
    /// A variable was passed where a literal was required. Reordering goals may fix this.
    #[fail(display = "Insufficiently instantiated arguments to {}/{}", _0, _1)]
    InsufficientlyInstantiatedArgs(&'static str, usize),

    /// A value of the wrong type was passed.
    #[fail(display = "Type error in arguments to {}/{}", _0, _1)]
    TypeError(&'static str, usize),
}
