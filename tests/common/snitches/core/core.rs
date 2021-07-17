use crate::common::snitches::Messages;
use common::async_trait::async_trait;
use models::{
    server::UdpTuple,
    transport::{RequestMsg, TransportMsg},
};
use sip_server::{CoreLayer, SipBuilder, SipManager, Transaction, Transport};
use std::any::Any;
use std::sync::{Arc, Weak};
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct CoreSnitch {
    sip_manager: Weak<SipManager>,
    pub messages: Messages,
}

#[async_trait]
impl CoreLayer for CoreSnitch {
    fn new(sip_manager: Weak<SipManager>) -> Self {
        Self {
            sip_manager: sip_manager.clone(),
            messages: Default::default(),
        }
    }

    async fn process_incoming_message(&self, msg: TransportMsg) {
        self.messages.push(msg).await;
    }

    async fn send(&self, msg: RequestMsg) -> Result<(), sip_server::Error> {
        match self.sip_manager().transport.send(msg.into()).await {
            Ok(_) => (),
            Err(err) => common::log::error!("failed to send message: {:?}", err),
        };

        Ok(())
    }

    async fn run(&self) {}

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl CoreSnitch {
    fn sip_manager(&self) -> Arc<SipManager> {
        self.sip_manager.upgrade().expect("sip manager is missing!")
    }
}

#[derive(Debug)]
pub struct CorePanic;

#[async_trait]
impl CoreLayer for CorePanic {
    fn new(sip_manager: Weak<SipManager>) -> Self {
        Self
    }

    async fn process_incoming_message(&self, msg: TransportMsg) {
        p!(self)
    }

    async fn send(&self, msg: RequestMsg) -> Result<(), sip_server::Error> {
        p!(self)
    }

    async fn run(&self) {}

    fn as_any(&self) -> &dyn Any {
        self
    }
}
