use crate::common::snitches::Messages;
use common::async_trait::async_trait;
use models::{server::UdpTuple, transport::TransportMsg};
use sip_server::{SipBuilder, SipManager, Transaction, Transport, TuLayer};
use std::any::Any;
use std::sync::{Arc, Weak};
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct UaSnitch {
    sip_manager: Weak<SipManager>,
    pub messages: Messages,
}

#[async_trait]
impl TuLayer for UaSnitch {
    fn new(sip_manager: Weak<SipManager>) -> Self {
        Self {
            sip_manager: sip_manager.clone(),
            messages: Default::default(),
        }
    }

    async fn process_incoming_message(&self, msg: TransportMsg) {
        self.messages.push(msg).await;
    }

    async fn send(&self, msg: TransportMsg) {
        match self.sip_manager().transport.send(msg).await {
            Ok(_) => (),
            Err(err) => common::log::error!("failed to send message: {:?}", err),
        }
    }

    async fn run(&self) {}

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl UaSnitch {
    fn sip_manager(&self) -> Arc<SipManager> {
        self.sip_manager.upgrade().expect("sip manager is missing!")
    }
}

#[derive(Debug)]
pub struct UaPanic;

#[async_trait]
impl TuLayer for UaPanic {
    fn new(sip_manager: Weak<SipManager>) -> Self {
        Self
    }

    async fn process_incoming_message(&self, msg: TransportMsg) {
        p!(self)
    }

    async fn send(&self, msg: TransportMsg) {
        p!(self)
    }

    async fn run(&self) {}

    fn as_any(&self) -> &dyn Any {
        self
    }
}
