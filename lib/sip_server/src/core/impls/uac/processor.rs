pub use crate::{presets, Error, ReqProcessor, SipManager};
use common::rsip::{self};
use models::transport::{RequestMsg, TransportMsg, ResponseMsg};
use std::sync::{Arc, Weak};

#[derive(Debug)]
pub struct UacProcessor {
    sip_manager: Weak<SipManager>,
}

impl UacProcessor {
    pub fn new(sip_manager: Weak<SipManager>) -> Self {
        Self { sip_manager }
    }

    pub async fn process_incoming_message(&self, msg: TransportMsg) -> Result<(), Error> {
        let sip_message = msg.sip_message;

        match sip_message {
            rsip::SipMessage::Request(request) => {
                self.handle_request(RequestMsg::new(request, msg.peer, msg.transport))
                    .await
            }
            rsip::SipMessage::Response(response) => {
                self.handle_response(ResponseMsg::new(response, msg.peer, msg.transport))
                    .await
            }
        }?;

        Ok(())
    }

    pub async fn handle_request(&self, _msg: RequestMsg) -> Result<(), Error> {
        Ok(())
    }

    pub async fn handle_response(&self, _msg: ResponseMsg) -> Result<(), Error> {
        Ok(())
    }

    fn sip_manager(&self) -> Arc<SipManager> {
        self.sip_manager.upgrade().expect("sip manager is missing!")
    }
}
