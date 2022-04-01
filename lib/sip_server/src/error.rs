#![allow(dead_code)]

use common::{
    rsip,
    tokio::sync::{mpsc::error::SendError, oneshot::error::RecvError},
};
use models::{transaction::TransactionLayerMsg, transport::TransportLayerMsg, tu::TuLayerMsg};
use std::{error::Error as StdError, fmt};

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
}

//this will be refactored that soonish
#[derive(Debug)]
pub enum ErrorKind {
    Empty,
    Models(models::Error),
    Rsip(rsip::Error),
    Custom(String),
    SipHelpers(String),
    Io(std::io::Error),
    Transaction(TransactionError),
    Dialog(DialogError),
    Channel(String),
    Store(store::Error),
}

#[derive(Debug)]
pub enum TransactionError {
    NotFound,
    UnexpectedState,
}

#[derive(Debug)]
pub enum DialogError {
    NotFound,
    UnexpectedState,
}

impl Error {
    pub fn custom(reason: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::from(reason.into()),
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

//TODO: fix me
impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorKind::Models(ref inner) => write!(f, "models transformation error: {}", inner),
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

impl From<models::Error> for ErrorKind {
    fn from(e: models::Error) -> Self {
        ErrorKind::Models(e)
    }
}

impl From<rsip::Error> for ErrorKind {
    fn from(e: rsip::Error) -> Self {
        ErrorKind::Rsip(e)
    }
}

impl From<std::io::Error> for ErrorKind {
    fn from(e: std::io::Error) -> Self {
        ErrorKind::Io(e)
    }
}

impl From<TransactionError> for ErrorKind {
    fn from(e: TransactionError) -> Self {
        ErrorKind::Transaction(e)
    }
}

impl From<DialogError> for ErrorKind {
    fn from(e: DialogError) -> Self {
        ErrorKind::Dialog(e)
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

impl From<store::Error> for ErrorKind {
    fn from(e: store::Error) -> Self {
        ErrorKind::Store(e)
    }
}
