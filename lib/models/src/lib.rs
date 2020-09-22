macro_rules! named_header_param {
    ($header:expr, $param:expr, $error:expr) => {
        if let Ok(header) = $header {
            if let Some(Some(param)) = header.parameters.get($param) {
                Ok(param)
            } else {
                Err($error)
            }
        } else {
            Err($error)
        }
    };
}

mod auth_request;
mod dialog;
mod error;
mod registration;
mod request;
mod response;
mod sip_message;
pub mod transactions;

pub mod core;
pub mod server;
pub mod transaction;
pub mod transport;

pub use auth_request::AuthRequest;
pub use dialog::{Dialog, DialogFlow};
pub use error::Error;
pub use registration::{Registration, UpdateRegistration};
pub use request::Request;
pub use response::Response;
pub use sip_message::SipMessage;

use tokio::sync::mpsc::{Receiver, Sender};

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum TransportType {
    Tcp,
    Udp,
}

pub type ChannelOf<T> = (Sender<T>, Receiver<T>);
