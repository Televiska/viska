//mod processor;

use crate::{presets, Dialogs, Error, ReqProcessor};
use common::{rsip, tokio};
use std::sync::Arc;

use models::{
    transport::{RequestMsg, ResponseMsg, TransportMsg},
    tu::TuLayerMsg,
    Handlers, TuReceiver,
};

//TODO: rename this to something else like ProxyTu etc
#[derive(Debug)]
pub struct UserAgent<R: ReqProcessor, C: ReqProcessor> {
    inner: Arc<Inner<R, C>>,
}

impl<R: ReqProcessor, C: ReqProcessor> UserAgent<R, C> {
    pub fn new(
        handlers: Handlers,
        messages_rx: TuReceiver,
        registrar: R,
        capabilities: C,
    ) -> Result<Self, Error> {
        let me = Self {
            inner: Arc::new(Inner {
                registrar,
                capabilities,
                dialogs: Dialogs::new(handlers.clone())?,
                handlers,
            }),
        };

        me.run(messages_rx);

        Ok(me)
    }

    fn run(&self, messages: TuReceiver) {
        let inner = self.inner.clone();
        tokio::spawn(async move { inner.run(messages).await });
    }
}

#[derive(Debug)]
struct Inner<R: ReqProcessor, C: ReqProcessor> {
    registrar: R,
    capabilities: C,
    dialogs: Dialogs,
    handlers: Handlers,
}

impl<R: ReqProcessor, C: ReqProcessor> Inner<R, C> {
    async fn run(&self, mut messages: TuReceiver) {
        while let Some(request) = messages.recv().await {
            if let Err(err) = self.receive(request.into()).await {
                common::log::error!("Error handling tu layer message: {}", err)
            }
        }
    }

    //TODO: here we don't spawn, could lead to deadlocks
    async fn receive(&self, msg: TuLayerMsg) -> Result<(), Error> {
        match msg {
            TuLayerMsg::Incoming(msg) => self.process_incoming_message(msg).await?,
        };

        Ok(())
    }

    async fn process_incoming_message(&self, msg: TransportMsg) -> Result<(), Error> {
        let sip_message = msg.sip_message;

        match sip_message {
            rsip::SipMessage::Request(request) => {
                self.handle_request(RequestMsg::new(request, msg.peer, msg.transport))
                    .await?;
            }
            rsip::SipMessage::Response(_) => common::log::error!("we don't support responses yet"),
        };

        Ok(())
    }

    async fn handle_request(&self, msg: RequestMsg) -> Result<(), Error> {
        use rsip::Method;

        match msg.sip_request.method {
            Method::Register => self.registrar.process_incoming_request(msg).await?,
            Method::Options => self.capabilities.process_incoming_request(msg).await?,
            _ => {
                self.handlers
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
}
