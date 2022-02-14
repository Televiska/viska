use crate::common::snitches::Messages;
use common::async_trait::async_trait;
use models::{server::UdpTuple, transport::RequestMsg};
use sip_server::{DialogsProcessor, Error, SipManager};
use std::any::Any;
use std::sync::{Arc, Weak};
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct DialogsEmptySnitch {
    sip_manager: Weak<SipManager>,
    //pub messages: Messages,
}

#[async_trait]
impl DialogsProcessor for DialogsEmptySnitch {
    fn new(sip_manager: Weak<SipManager>) -> Self {
        Self {
            sip_manager: sip_manager.clone(),
            //messages: Default::default(),
        }
    }

    async fn has_dialog(&self, dialog_id: &str) -> bool {
        false
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
