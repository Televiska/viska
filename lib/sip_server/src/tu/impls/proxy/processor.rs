pub use crate::{presets, Error, ReqProcessor, SipManager, TuProcessor};
use common::{
    async_trait::async_trait,
    rsip::{self},
};
//use models::transport::ResponseMsg;
use models::transport::{RequestMsg, TransportMsg};
use std::{
    any::Any,
    sync::{Arc, Weak},
};

#[derive(Debug)]
pub struct ProxyProcessor {
    sip_manager: Weak<SipManager>,
}

#[async_trait]
impl TuProcessor for ProxyProcessor {
    fn new(sip_manager: Weak<SipManager>) -> Self {
        Self { sip_manager }
    }

    async fn process_incoming_message(&self, msg: TransportMsg) -> Result<(), Error> {
        let sip_message = msg.sip_message;

        match sip_message {
            rsip::SipMessage::Request(request) => {
                self.handle_request(RequestMsg::new(request, msg.peer, msg.transport))
                    .await
            }
            rsip::SipMessage::Response(_) => Err(Error::from("we don't support responses yet")),
        }?;

        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl ProxyProcessor {
    async fn handle_request(&self, _msg: RequestMsg) -> Result<(), Error> {
        Err(Error::from("we don't support requests yet"))
    }

    fn sip_manager(&self) -> Arc<SipManager> {
        self.sip_manager.upgrade().expect("sip manager is missing!")
    }
}
