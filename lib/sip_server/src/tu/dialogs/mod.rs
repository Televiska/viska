mod uac;
mod uas;

pub use crate::Error;
use common::tokio::sync::{Mutex, RwLock};
use models::Handlers;
use std::collections::HashMap;
use std::sync::Arc;

//TODO: why inner/Arc ?
#[derive(Debug)]
pub struct Dialogs {
    #[allow(dead_code)]
    inner: Arc<Inner>,
}

#[derive(Debug)]
struct Inner {
    #[allow(dead_code)]
    pub uac_state: RwLock<HashMap<String, Mutex<uac::DialogSm>>>,
    #[allow(dead_code)]
    pub uas_state: RwLock<HashMap<String, Mutex<uas::DialogSm>>>,
    handlers: Handlers,
}

impl Dialogs {
    pub fn new(handlers: Handlers) -> Result<Self, Error> {
        let inner = Arc::new(Inner {
            handlers,
            uac_state: RwLock::new(Default::default()),
            uas_state: RwLock::new(Default::default()),
        });

        Ok(Self { inner })
    }

    pub async fn has_dialog(&self, _dialog_id: &str) -> bool {
        false
    }
}
