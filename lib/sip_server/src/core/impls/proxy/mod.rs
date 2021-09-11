mod processor;

use crate::CoreProcessor;
use common::{async_trait::async_trait};
use std::{
    any::Any,
    sync::{Weak},
    fmt::Debug
};

use crate::SipManager;
use models::transport::{RequestMsg, ResponseMsg};

pub type Proxy = crate::core::impls::UserAgent<dyn CoreProcessor>;

#[async_trait]
pub trait ProxyProcessor: Send + Sync + Any + Debug {
    fn new(sip_manager: Weak<SipManager>) -> Self
    where
        Self: Sized;
    async fn validate_request(&self, msg: RequestMsg) -> Result<(), crate::Error>;
    async fn preprocess_routing_info(&self, msg: RequestMsg) -> Result<(), crate::Error>;
    async fn determine_targets(&self, msg: RequestMsg) -> Result<(), crate::Error>;
    async fn forward_request(&self, msg: RequestMsg) -> Result<(), crate::Error>;
    async fn process_response(&self, msg: ResponseMsg) -> Result<(), crate::Error>;
    fn as_any(&self) -> &dyn Any;
}
