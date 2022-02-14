use crate::common::snitches::Messages;
use common::async_trait::async_trait;
use models::{server::UdpTuple, transport::RequestMsg};
use sip_server::{ReqProcessor, Error, SipManager};
use std::any::Any;
use std::sync::{Arc, Weak};
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct CapabilitiesSnitch {
    sip_manager: Weak<SipManager>,
    pub messages: Messages,
}

#[async_trait]
impl ReqProcessor for CapabilitiesSnitch {
    fn new(sip_manager: Weak<SipManager>) -> Self {
        Self {
            sip_manager: sip_manager.clone(),
            messages: Default::default(),
        }
    }

    async fn process_incoming_request(&self, msg: RequestMsg) -> Result<(), Error> {
        self.messages.push(msg.into()).await;

        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
