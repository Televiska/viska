macro_rules! header {
    ($iter:expr, $header:path, $error:expr) => {
        $iter
            .find_map(|header| {
                if let $header(header) = header {
                    Some(header)
                } else {
                    None
                }
            })
            .ok_or($error)
    };
}

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

macro_rules! named_header_username {
    ($header:expr, $error:expr) => {
        if let Ok(header) = $header {
            if let Some(auth) = &header.uri.auth {
                Ok(&auth.username)
            } else {
                Err($error)
            }
        } else {
            Err($error)
        }
    };
}

mod dialog;
mod registration;
mod request;
mod response;
mod transactions;

pub use dialog::Dialog;
pub use registration::Registration;
pub use request::Request;
pub use response::Response;

pub struct ServerState {
    pub request: Request,
    pub dialog: Dialog,
}

pub trait TransactionFSM {
    fn next(&self, request: crate::Request) -> Result<Response, String>;
}

pub trait DialogExt {
    fn transaction(&self) -> Box<dyn crate::TransactionFSM>;
}
