use super::Messages;
use common::async_trait::async_trait;
use models::{server::UdpTuple, transport::TransportMsg};
use sip_server::{Error, SipManager, TransportLayer};
use std::any::Any;
use std::sync::{Arc, Weak};
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct TransportSnitch {
    sip_manager: Weak<SipManager>,
    pub messages: Messages,
}

#[async_trait]
impl TransportLayer for TransportSnitch {
    fn new(sip_manager: Weak<SipManager>) -> Result<Self, Error> {
        Ok(Self {
            sip_manager: sip_manager.clone(),
            messages: Default::default(),
        })
    }

    async fn process_incoming_message(&self, _: UdpTuple) -> Result<(), Error> {
        Ok(())
    }

    async fn send(&self, msg: TransportMsg) -> Result<(), Error> {
        self.messages.push(msg).await;

        Ok(())
    }

    async fn run(&self) {}

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug)]
pub struct TransportErrorSnitch {
    sip_manager: Weak<SipManager>,
    fail_switch: Mutex<bool>,
    pub messages: Messages,
}

#[async_trait]
impl TransportLayer for TransportErrorSnitch {
    fn new(sip_manager: Weak<SipManager>) -> Result<Self, Error> {
        Ok(Self {
            sip_manager: sip_manager.clone(),
            fail_switch: Mutex::new(true),
            messages: Default::default(),
        })
    }

    async fn process_incoming_message(&self, _: UdpTuple) -> Result<(), Error> {
        match *self.fail_switch.lock().await {
            true => Err("this is just a snitch".into()),
            false => Ok(()),
        }
    }

    async fn send(&self, msg: TransportMsg) -> Result<(), Error> {
        match *self.fail_switch.lock().await {
            true => Err("this is just a snitch".into()),
            false => {
                self.messages.push(msg).await;
                Ok(())
            }
        }
    }

    async fn run(&self) {}

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl TransportErrorSnitch {
    pub async fn turn_fail_switch_off(&self) {
        let mut switch = self.fail_switch.lock().await;
        *switch = false;
    }

    pub async fn turn_fail_switch_on(&self) {
        let mut switch = self.fail_switch.lock().await;
        *switch = true;
    }
}

#[derive(Debug)]
pub struct TransportPanic;

#[async_trait]
impl TransportLayer for TransportPanic {
    fn new(sip_manager: Weak<SipManager>) -> Result<Self, Error> {
        Ok(Self)
    }

    async fn process_incoming_message(&self, _: UdpTuple) -> Result<(), Error> {
        p!(self)
    }

    async fn send(&self, msg: TransportMsg) -> Result<(), Error> {
        p!(self)
    }

    async fn run(&self) {}

    fn as_any(&self) -> &dyn Any {
        self
    }
}
