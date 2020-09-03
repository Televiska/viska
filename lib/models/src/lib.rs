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
pub mod transactions;
mod sip_message;

pub use sip_message::SipMessage;
pub use auth_request::AuthRequest;
pub use dialog::{Dialog, DialogFlow};
pub use error::Error;
pub use registration::{Registration, UpdateRegistration};
pub use request::Request;
pub use response::Response;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum TransportType {
    Tcp,
    Udp,
}
