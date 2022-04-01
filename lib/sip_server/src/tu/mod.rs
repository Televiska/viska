pub mod dialogs;
pub mod elements;

use common::{async_trait::async_trait, rsip};
use std::fmt::Debug;

#[async_trait]
pub trait ReqProcessor: Send + Sync + Debug + 'static {
    async fn process_incoming_request(&self, msg: rsip::Request) -> Result<(), crate::Error>;
}

#[async_trait]
pub trait RespProcessor: Send + Sync + Debug + 'static {
    async fn process_incoming_response(&self, msg: rsip::Response) -> Result<(), crate::Error>;
}

/*
#[async_trait]
pub trait DialogsProcessor: Send + Sync + Any + Debug {
    fn new(sip_manager: Weak<SipManager>) -> Self
    where
        Self: Sized;
    async fn has_dialog(&self, dialog_id: &str) -> bool;
    async fn new_uac_dialog(&self, msg: RequestMsg) -> Result<(), crate::Error>;
    async fn new_uas_dialog(&self, msg: RequestMsg) -> Result<(), crate::Error>;
    async fn process_incoming_message(&self, msg: RequestMsg) -> Result<(), crate::Error>;
    fn as_any(&self) -> &dyn Any;
}*/

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
