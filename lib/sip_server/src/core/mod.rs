mod capabilities;
#[allow(clippy::module_inception)]
mod core;
mod dialogs;
mod processor;
mod registrar;

pub use self::core::Core;
pub use capabilities::Capabilities;
pub use dialogs::Dialogs;
pub use processor::Processor;
pub use registrar::Registrar;

use common::async_trait::async_trait;
use std::{any::Any, fmt::Debug, sync::Weak};

use crate::{Error, SipManager};
use models::transport::{RequestMsg, TransportMsg};

#[async_trait]
pub trait CoreLayer: Send + Sync + Any + Debug {
    fn new(sip_manager: Weak<SipManager>) -> Self
    where
        Self: Sized;
    async fn process_incoming_message(&self, msg: TransportMsg);
    async fn send(&self, request: rsip::Request) -> Result<(), Error>;
    async fn run(&self);
    fn as_any(&self) -> &dyn Any;
}

#[async_trait]
pub trait CoreProcessor: Send + Sync + Any + Debug {
    fn new(sip_manager: Weak<SipManager>) -> Self
    where
        Self: Sized;
    async fn process_incoming_message(&self, msg: TransportMsg) -> Result<(), Error>;
    async fn send(&self, request: rsip::Request) -> Result<(), Error>;
    fn as_any(&self) -> &dyn Any;
}

#[async_trait]
pub trait ReqProcessor: Send + Sync + Any + Debug {
    fn new(sip_manager: Weak<SipManager>) -> Self
    where
        Self: Sized;
    async fn process_incoming_request(&self, msg: RequestMsg) -> Result<(), Error>;
    fn as_any(&self) -> &dyn Any;
}

pub trait DialogsProcessor: Send + Sync + Any + Debug {
    fn new(sip_manager: Weak<SipManager>) -> Self
    where
        Self: Sized;
    async fn has_dialog(&self, dialog_id: &str) -> bool;
    fn as_any(&self) -> &dyn Any;
}
