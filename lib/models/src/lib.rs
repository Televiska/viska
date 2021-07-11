mod auth_request;
pub mod core;
mod dialog;
mod error;
pub mod server;
mod sip_message_ext;
pub mod transaction;
pub mod transactions;
pub mod transport;

pub use auth_request::AuthRequest;
pub use dialog::{Dialog, DialogFlow};
pub use error::Error;
pub use sip_message_ext::RequestExt;

use tokio::sync::mpsc::{Receiver, Sender};

pub type ChannelOf<T> = (Sender<T>, Receiver<T>);
