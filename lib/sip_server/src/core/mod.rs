pub mod impls;
mod dialogs;

pub use dialogs::Dialogs;

use common::async_trait::async_trait;
use std::{any::Any, fmt::Debug, sync::Weak};

use crate::SipManager;
use models::transport::{RequestMsg, ResponseMsg, TransportMsg};

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

#[async_trait]
pub trait RespProcessor: Send + Sync + Any + Debug {
    fn new(sip_manager: Weak<SipManager>) -> Self
    where
        Self: Sized;
    async fn process_incoming_response(&self, msg: ResponseMsg) -> Result<(), crate::Error>;
    fn as_any(&self) -> &dyn Any;
}

#[async_trait]
pub trait DialogsProcessor: Send + Sync + Any + Debug {
    fn new(sip_manager: Weak<SipManager>) -> Self
    where
        Self: Sized;
    async fn has_dialog(&self, dialog_id: &str) -> bool;
    fn as_any(&self) -> &dyn Any;
}

//#[async_trait]
//pub trait ProxyProcessor: Send + Sync + Any + Debug {
//    fn new(sip_manager: Weak<SipManager>) -> Self
//    where
//        Self: Sized;
//    async fn validate_request(&self, msg: RequestMsg) -> Result<(), crate::Error>;
//    async fn preprocess_routing_info(&self, msg: RequestMsg) -> Result<(), crate::Error>;
//    async fn determine_targets(&self, msg: RequestMsg) -> Result<(), crate::Error>;
//    async fn forward_request(&self, msg: RequestMsg) -> Result<(), crate::Error>;
//    async fn process_response(&self, msg: ResponseMsg) -> Result<(), crate::Error>;
//    fn as_any(&self) -> &dyn Any;
//}
