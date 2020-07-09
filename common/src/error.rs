use std::{error::Error as StdError, fmt};

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
}

#[derive(Debug)]
pub enum ErrorKind {
    Empty,
    IO(std::io::Error),
    Custom(String),
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorKind::Custom(ref inner) => write!(f, "error: {}", inner),
            _ => write!(f, "unknown error, {:?}", self),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unexpected error: {}", self.kind)
    }
}

impl StdError for Error {}

impl<E> From<E> for Error
where
    E: Into<ErrorKind>,
{
    fn from(e: E) -> Self {
        Error { kind: e.into() }
    }
}

impl From<std::io::Error> for ErrorKind {
    fn from(e: std::io::Error) -> Self {
        ErrorKind::IO(e)
    }
}

impl From<String> for ErrorKind {
    fn from(e: String) -> Self {
        ErrorKind::Custom(e)
    }
}

impl From<&str> for ErrorKind {
    fn from(e: &str) -> Self {
        ErrorKind::Custom(e.into())
    }
}
