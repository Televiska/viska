//mod processor;

use crate::{presets, tu::dialogs::Dialogs, Error, ReqProcessor};
use common::{rsip, tokio};
use std::sync::Arc;

use models::{receivers::TuReceiver, tu::TuLayerMsg, Handlers};

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
                dialogs: Dialogs::new(handlers.clone()),
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
    #[allow(dead_code)]
    dialogs: Dialogs,
    handlers: Handlers,
}

impl<R: ReqProcessor, C: ReqProcessor> Inner<R, C> {
    async fn run(&self, mut messages: TuReceiver) {
        while let Some(request) = messages.recv().await {
            if let Err(err) = self.receive(request).await {
                common::log::error!("Error handling tu layer message: {}", err)
            }
        }
    }

    //TODO: here we don't spawn, could lead to deadlocks
    async fn receive(&self, msg: TuLayerMsg) -> Result<(), Error> {
        match msg {
            TuLayerMsg::Incoming(msg) => self.process_incoming_message(msg).await?,
            TuLayerMsg::Outgoing(msg) => self.process_outgoing_message(msg).await?,
            TuLayerMsg::TransportError(msg, error) => {
                self.process_transport_error(msg, error).await?
            }
        };

        Ok(())
    }

    async fn process_transport_error(
        &self,
        msg: rsip::SipMessage,
        error: String,
    ) -> Result<(), Error> {
        Ok(self.dialogs.transport_error(msg, error).await?)
    }

    async fn process_incoming_message(&self, msg: rsip::SipMessage) -> Result<(), Error> {
        match msg {
            rsip::SipMessage::Request(request) => {
                self.handle_incoming_request(request).await?;
            }
            rsip::SipMessage::Response(response) => {
                self.handle_incoming_response(response).await?;
            }
        };

        Ok(())
    }

    async fn process_outgoing_message(&self, msg: rsip::SipMessage) -> Result<(), Error> {
        match msg {
            rsip::SipMessage::Request(request) => {
                self.handle_outgoing_request(request).await?;
            }
            rsip::SipMessage::Response(response) => {
                self.handle_outgoing_response(response).await?;
            }
        };

        Ok(())
    }

    async fn handle_incoming_request(&self, request: rsip::Request) -> Result<(), Error> {
        use rsip::Method;

        match request.method {
            Method::Register => self.registrar.process_incoming_request(request).await?,
            Method::Options => self.capabilities.process_incoming_request(request).await?,
            _ => {
                self.handlers
                    .transport
                    .send(presets::create_405_from(request)?.into())
                    .await?
            }
        };

        Ok(())
    }

    async fn handle_incoming_response(&self, response: rsip::Response) -> Result<(), Error> {
        if let Ok(dialog_id) = response.dialog_id() {
            if self.dialogs.exists(dialog_id).await {
                //TODO: this is wrong, uac dialogs can process requests as well
                self.dialogs.uac_process_incoming_response(response).await?
            } else {
                common::log::warn!("received response msg but no dialog exists for that msg");
            };
        }

        Ok(())
    }

    async fn handle_outgoing_request(&self, request: rsip::Request) -> Result<(), Error> {
        use rsip::Method;

        match request.method {
            Method::Invite => {
                //TODO: consider letting the dialog handle the transaction creation ?
                self.dialogs.new_uac_session(request.clone()).await?;
            }
            _ => self.handlers.transport.send(request.into()).await?,
        };

        Ok(())
    }

    async fn handle_outgoing_response(&self, response: rsip::Response) -> Result<(), Error> {
        self.handlers.transport.send(response.into()).await?;

        Ok(())
    }
}
