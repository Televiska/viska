#![allow(dead_code)]

use common::{
    ipnetwork, rsip,
    tokio::sync::{mpsc::error::SendError, oneshot::error::RecvError},
};
use std::{error::Error as StdError, fmt};
use crate::{transaction::TransactionLayerMsg, transport::TransportLayerMsg, tu::TuLayerMsg};

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
}

//this will be refactored that soonish
#[derive(Debug)]
pub enum ErrorKind {
    Empty,
    IpAddress(String),
    Rsip(rsip::Error),
    Channel(String),
    Custom(String),
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

impl From<ipnetwork::IpNetworkError> for ErrorKind {
    fn from(e: ipnetwork::IpNetworkError) -> Self {
        ErrorKind::IpAddress(e.to_string())
    }
}

impl From<rsip::Error> for ErrorKind {
    fn from(e: rsip::Error) -> Self {
        ErrorKind::Rsip(e)
    }
}

impl From<SendError<TransportLayerMsg>> for ErrorKind {
    fn from(e: SendError<TransportLayerMsg>) -> Self {
        ErrorKind::Channel(e.to_string())
    }
}

impl From<SendError<TransactionLayerMsg>> for ErrorKind {
    fn from(e: SendError<TransactionLayerMsg>) -> Self {
        ErrorKind::Channel(e.to_string())
    }
}

impl From<SendError<TuLayerMsg>> for ErrorKind {
    fn from(e: SendError<TuLayerMsg>) -> Self {
        ErrorKind::Channel(e.to_string())
    }
}

impl From<RecvError> for ErrorKind {
    fn from(e: RecvError) -> Self {
        ErrorKind::Channel(e.to_string())
    }
}
