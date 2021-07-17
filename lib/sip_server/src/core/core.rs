pub use super::{Capabilities, CoreLayer, DialogsProcessor, Registrar, ReqProcessor};
use common::{
    async_trait::async_trait,
    rsip::{self, prelude::*},
};
use std::{
    any::Any,
    sync::{Arc, Weak},
};

pub use crate::{presets, Error, SipManager};
use models::transport::{RequestMsg, ResponseMsg, TransportMsg};

//TODO: rename this to something else like ProxyCore etc
pub struct Core<R: ReqProcessor, C: ReqProcessor, D: DialogsProcessor> {
    inner: Arc<Inner<R, C, D>>,
}

#[async_trait]
impl<R: ReqProcessor, C: ReqProcessor, D: DialogsProcessor> CoreLayer for Core<R, C, D> {
    fn new(sip_manager: Weak<SipManager>) -> Self {
        let inner = Arc::new(Inner::new(sip_manager.clone()));
        Self { inner }
    }

    async fn process_incoming_message(&self, msg: TransportMsg) {
        self.inner.process_incoming_message(msg).await
    }

    async fn send(&self, msg: RequestMsg) -> Result<(), Error> {
        Ok(self.inner.send(msg).await?)
    }

    //TODO: fix me
    async fn run(&self) {}

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug)]
pub struct Inner<R: ReqProcessor, C: ReqProcessor, D: DialogsProcessor> {
    sip_manager: Weak<SipManager>,
    registrar: R,
    capabilities: C,
    dialogs: D,
}

#[async_trait]
impl<R: ReqProcessor, C: ReqProcessor, D: DialogsProcessor> CoreLayer for Inner<R, C, D> {
    fn new(sip_manager: Weak<SipManager>) -> Self {
        Self {
            registrar: R::new(sip_manager.clone()),
            capabilities: C::new(sip_manager.clone()),
            dialogs: D::new(sip_manager.clone()),
            sip_manager,
        }
    }

    async fn process_incoming_message(&self, msg: TransportMsg) {
        let sip_message = msg.sip_message;

        //TODO: fix me
        match match sip_message {
            rsip::SipMessage::Request(request) => {
                self.handle_request(RequestMsg::new(request, msg.peer, msg.transport))
                    .await
            }
            rsip::SipMessage::Response(_) => Err(Error::from("we don't support responses yet")),
        } {
            Ok(_) => (),
            Err(err) => common::log::warn!("failed to process message: {:?}", err),
        }
    }

    async fn send(&self, _msg: RequestMsg) -> Result<(), Error> {
        Ok(())
    }

    async fn run(&self) {}

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl<R: ReqProcessor, C: ReqProcessor, D: DialogsProcessor> Inner<R, C, D> {
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

impl<R: ReqProcessor, C: ReqProcessor, D: DialogsProcessor> std::fmt::Debug for Core<R, C, D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Core")
            .field("processor", &self.inner)
            .finish()
    }
}
