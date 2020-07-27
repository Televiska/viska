#![allow(dead_code)]

use std::{error::Error as StdError, fmt};

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
}

//this will be refactored that soonish
#[derive(Debug)]
pub enum ErrorKind {
    Empty,
    Processor(processor::Error),
    Custom(String),
    Io(std::io::Error),
}

impl Error {
    pub fn custom(reason: String) -> Self {
        Self {
            kind: ErrorKind::from(reason),
        }
    }
}

impl From<Option<ErrorKind>> for ErrorKind {
    fn from(kind: Option<ErrorKind>) -> Self {
        match kind {
            None => ErrorKind::Empty,
            Some(kind) => kind,
        }
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorKind::Processor(ref inner) => write!(f, "Processor error: {}", inner),
            ErrorKind::Custom(ref inner) => write!(f, "{}", inner),
            _ => write!(f, "unknown error, {:?}", self),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.kind)
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

impl From<processor::Error> for ErrorKind {
    fn from(e: processor::Error) -> Self {
        ErrorKind::Processor(e)
    }
}

impl From<std::io::Error> for ErrorKind {
    fn from(e: std::io::Error) -> Self {
        ErrorKind::Io(e)
    }
}
