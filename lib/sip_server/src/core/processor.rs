pub use super::{Capabilities, Registrar};
pub use crate::{presets, Error, SipManager};
use models::transport::ResponseMsg;
use models::transport::{RequestMsg, TransportMsg};
use rsip::SipMessage;
use std::sync::{Arc, Weak};

#[derive(Debug)]
pub struct Processor {
    sip_manager: Weak<SipManager>,
    registrar: Registrar,
    capabilities: Capabilities,
}

#[allow(clippy::new_without_default)]
impl Processor {
    pub fn new(sip_manager: Weak<SipManager>) -> Self {
        Self {
            registrar: Registrar::new(sip_manager.clone()),
            capabilities: Capabilities::new(sip_manager.clone()),
            sip_manager,
        }
    }

    //TODO: Fix me
    pub async fn process_message(&self, msg: TransportMsg) -> Result<(), Error> {
        let sip_message = msg.sip_message;

        match sip_message {
            SipMessage::Request(request) => {
                self.handle_request(RequestMsg::new(request, msg.peer, msg.transport))
                    .await
            }
            SipMessage::Response(_) => Err(Error::from("we don't support responses yet")),
        }?;

        Ok(())
    }

    async fn handle_request(&self, msg: RequestMsg) -> Result<(), Error> {
        use rsip::common::Method;

        match msg.sip_request.method {
            Method::Register => self.registrar.process_incoming_request(msg).await?,
            Method::Options => self.capabilities.process_incoming_request(msg).await?,
            _ => {
                self.sip_manager()
                    .transport
                    .send(
                        ResponseMsg::new(
                            presets::create_405_from(msg.sip_request)?,
                            msg.peer,
                            msg.transport,
                        )
                        .into(),
                    )
                    .await?
            }
        };

        Ok(())
    }

    fn sip_manager(&self) -> Arc<SipManager> {
        self.sip_manager.upgrade().expect("sip manager is missing!")
    }
}
