macro_rules! header {
    ($iter:expr, $header:path) => {
        $iter.find_map(|header| {
            if let $header(header) = header {
                Some(header)
            } else {
                None
            }
        })
    };
}

macro_rules! named_header_param {
    ($header:expr, $param:expr) => {
        $header.and_then(|header| {
            if let Some(param) = header.parameters.get($param) {
                param.as_ref()
            } else {
                None
            }
        })
    };
}

mod dialog;
mod not_found;
mod request;
mod response;
mod transaction;

pub use dialog::{Dialog, TransactionType};
pub use not_found::NotFound;
pub use request::Request;
pub use response::Response;
pub use transaction::Transaction;
pub struct ServerState {
    pub request: Request,
    pub dialog: Dialog,
}
