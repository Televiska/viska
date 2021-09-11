pub use crate::{presets, Error, SipManager, CoreProcessor, ReqProcessor};
use common::{
    async_trait::async_trait,
    rsip::{self, prelude::*},
};
use models::transport::ResponseMsg;
use models::transport::{RequestMsg, TransportMsg};
use std::{
    any::Any,
    sync::{Arc, Weak},
};

#[derive(Debug)]
pub struct UasProcessor<R: ReqProcessor, C: ReqProcessor> {
    sip_manager: Weak<SipManager>,
    registrar: R,
    capabilities: C,
}

#[async_trait]
impl<R: ReqProcessor, C: ReqProcessor> CoreProcessor for UasProcessor<R, C> {
    fn new(sip_manager: Weak<SipManager>) -> Self {
        Self {
            registrar: R::new(sip_manager.clone()),
            capabilities: C::new(sip_manager.clone()),
            sip_manager,
        }
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

impl<R: ReqProcessor, C: ReqProcessor> UasProcessor<R, C> {
    async fn handle_request(&self, msg: RequestMsg) -> Result<(), Error> {
        use rsip::Method;

        match msg.sip_request.method {
            Method::Register => {
                self.registrar
                    .process_incoming_request(self.with_auth(msg).await?)
                    .await?
            }
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

    async fn with_auth(&self, msg: RequestMsg) -> Result<RequestMsg, Error> {
        match msg.sip_request.authorization_header() {
            Some(_) => Ok(msg),
            None => {
                self.sip_manager()
                    .transport
                    .send(
                        ResponseMsg::from((
                            presets::create_unauthorized_from(msg.sip_request)?,
                            msg.peer,
                            msg.transport,
                        ))
                        .into(),
                    )
                    .await?;
                Err(Error::from("missing auth header"))
            }
        }
    }
}
