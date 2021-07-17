mod uac;
mod uas;

use super::DialogsProcessor;
pub use crate::{Error, SipManager};
use common::async_trait::async_trait;
use common::tokio::sync::{Mutex, RwLock};
use std::collections::HashMap;
use std::{
    any::Any,
    sync::{Arc, Weak},
};

#[derive(Debug)]
pub struct Dialogs {
    inner: Arc<Inner>,
}

#[derive(Debug)]
struct Inner {
    sip_manager: Weak<SipManager>,
    pub uac_state: RwLock<HashMap<String, Mutex<uac::DgStateMachine>>>,
    pub uas_state: RwLock<HashMap<String, Mutex<uas::DgStateMachine>>>,
}

#[async_trait]
impl DialogsProcessor for Dialogs {
    fn new(sip_manager: Weak<SipManager>) -> Self {
        let inner = Arc::new(Inner {
            sip_manager,
            uac_state: RwLock::new(Default::default()),
            uas_state: RwLock::new(Default::default()),
        });

        Self { inner }
    }

    async fn has_dialog(&self, _dialog_id: &str) -> bool {
        false
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[allow(dead_code)]
impl Inner {
    fn sip_manager(&self) -> Arc<SipManager> {
        self.sip_manager.upgrade().expect("sip manager is missing!")
    }
}
