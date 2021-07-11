use super::Messages;
use common::{async_trait::async_trait, rsip::prelude::*};
use models::transport::{RequestMsg, ResponseMsg, TransportMsg};
use sip_server::{Error, SipManager, TransactionLayer};
use std::any::Any;
use std::sync::{Arc, Weak};
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct TransactionEmptySnitch {
    sip_manager: Weak<SipManager>,
}

#[async_trait]
impl TransactionLayer for TransactionEmptySnitch {
    fn new(sip_manager: Weak<SipManager>) -> Self {
        Self {
            sip_manager: sip_manager.clone(),
        }
    }

    async fn new_uac_invite_transaction(&self, _: RequestMsg) -> Result<(), Error> {
        Err(Error::custom("unimplemented"))
    }

    async fn new_uas_invite_transaction(
        &self,
        _: RequestMsg,
        _: Option<rsip::Response>,
    ) -> Result<(), Error> {
        Err(Error::custom("unimplemented"))
    }

    async fn has_transaction(&self, _: &str) -> bool {
        false
    }

    async fn process_incoming_message(&self, _: TransportMsg) {}

    async fn send(&self, _: ResponseMsg) -> Result<(), Error> {
        Ok(())
    }

    fn sip_manager(&self) -> Arc<SipManager> {
        self.sip_manager.upgrade().expect("sip manager is missing!")
    }

    async fn run(&self) {
        ();
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug)]
pub struct TransactionPanic;

#[async_trait]
impl TransactionLayer for TransactionPanic {
    fn new(sip_manager: Weak<SipManager>) -> Self {
        Self
    }

    async fn new_uac_invite_transaction(&self, _: RequestMsg) -> Result<(), Error> {
        panic!("{:?} panics", self)
    }

    async fn new_uas_invite_transaction(
        &self,
        _: RequestMsg,
        _: Option<rsip::Response>,
    ) -> Result<(), Error> {
        p!(self)
    }

    async fn has_transaction(&self, _: &str) -> bool {
        p!(self)
    }

    async fn process_incoming_message(&self, _: TransportMsg) {
        p!(self)
    }

    async fn send(&self, _: ResponseMsg) -> Result<(), Error> {
        p!(self)
    }

    fn sip_manager(&self) -> Arc<SipManager> {
        p!(self)
    }

    async fn run(&self) {
        ();
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
