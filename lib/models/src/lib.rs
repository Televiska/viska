mod auth_request;
mod error;
pub mod handlers;
pub mod receivers;
pub mod result_ext;
pub mod rsip_ext;
pub mod transaction;
pub mod transport;
pub mod tu;

pub use auth_request::AuthRequest;
pub use error::Error;
pub use handlers::Handlers;
pub use result_ext::ResultExt;

use common::tokio::sync::mpsc::channel;

pub fn channels_builder() -> (Handlers, receivers::Receivers) {
    let (tu_tx, tu_rx) = channel(10);
    let (transaction_tx, transaction_rx) = channel(10);
    let (transport_tx, transport_rx) = channel(10);

    let handlers = (tu_tx, transaction_tx, transport_tx).into();

    let receivers = (tu_rx, transaction_rx, transport_rx).into();

    (handlers, receivers)
}
