#![allow(dead_code)]

use common::{ipnetwork, libsip};
use std::{error::Error as StdError, fmt};

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
}

//this will be refactored that soonish
#[derive(Debug)]
pub enum ErrorKind {
    Empty,
    Libsip(String),
    IpAddress(String),
    Custom(String),
}

impl Error {
    pub fn libsip(reason: String) -> Self {
        Self {
            kind: ErrorKind::Libsip(reason),
        }
    }
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
            ErrorKind::Libsip(ref inner) => write!(f, "Libsip error: {}", inner),
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

impl From<libsip::core::SipMessageError> for ErrorKind {
    fn from(e: libsip::core::SipMessageError) -> Self {
        ErrorKind::Libsip(format!("{:?}", e))
    }
}

impl From<ipnetwork::IpNetworkError> for ErrorKind {
    fn from(e: ipnetwork::IpNetworkError) -> Self {
        ErrorKind::IpAddress(e.to_string())
    }
}
