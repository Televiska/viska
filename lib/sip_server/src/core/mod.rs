mod capabilities;
#[allow(clippy::module_inception)]
mod core;
mod processor;
mod registrar;

pub use self::core::Core;
pub use capabilities::Capabilities;
pub use processor::Processor;
pub use registrar::Registrar;

use common::async_trait::async_trait;
use std::{any::Any, fmt::Debug, sync::Weak};

use crate::SipManager;
use models::transport::{RequestMsg, TransportMsg};

#[async_trait]
pub trait CoreLayer: Send + Sync + Any + Debug {
    fn new(sip_manager: Weak<SipManager>) -> Self
    where
        Self: Sized;
    async fn process_incoming_message(&self, msg: TransportMsg);
    async fn send(&self, msg: TransportMsg);
    async fn run(&self);
    fn as_any(&self) -> &dyn Any;
}

#[async_trait]
pub trait CoreProcessor: Send + Sync + Any + Debug {
    fn new(sip_manager: Weak<SipManager>) -> Self
    where
        Self: Sized;
    async fn process_incoming_message(&self, msg: TransportMsg) -> Result<(), crate::Error>;
    fn as_any(&self) -> &dyn Any;
}

#[async_trait]
pub trait ReqProcessor: Send + Sync + Any + Debug {
    fn new(sip_manager: Weak<SipManager>) -> Self
    where
        Self: Sized;
    async fn process_incoming_request(&self, msg: RequestMsg) -> Result<(), crate::Error>;
    fn as_any(&self) -> &dyn Any;
}
