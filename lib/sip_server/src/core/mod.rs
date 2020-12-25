pub mod processor;
use common::async_trait::async_trait;
use std::{
    any::Any,
    fmt::Debug,
    sync::{Arc, Weak},
};

use crate::SipManager;
use models::transport::TransportMsg;

#[async_trait]
pub trait CoreLayer: Send + Sync + Any + Debug {
    fn new(sip_manager: Weak<SipManager>) -> Self
    where
        Self: Sized;
    async fn process_incoming_message(&self, msg: TransportMsg);
    async fn send(&self, msg: TransportMsg);
    fn sip_manager(&self) -> Arc<SipManager>;
    fn as_any(&self) -> &dyn Any;
}

pub struct Core {
    sip_manager: Weak<SipManager>,
    processor: Arc<processor::Processor>,
}

#[async_trait]
impl CoreLayer for Core {
    fn new(sip_manager: Weak<SipManager>) -> Self {
        Self {
            sip_manager: sip_manager.clone(),
            processor: Arc::new(processor::Processor::new(sip_manager)),
        }
    }

    async fn process_incoming_message(&self, msg: TransportMsg) {
        let processor = self.processor.clone();
        tokio::spawn(async move {
            processor
                .process_message(msg)
                .await
                .expect("process message");
        });
    }

    async fn send(&self, msg: TransportMsg) {
        match self.sip_manager().transport.send(msg).await {
            Ok(_) => (),
            Err(err) => common::log::error!("failed to send message: {:?}", err),
        }
    }

    fn sip_manager(&self) -> Arc<SipManager> {
        self.sip_manager.upgrade().expect("sip manager is missing!")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl std::fmt::Debug for Core {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Core")
            .field("processor", &self.processor)
            .finish()
    }
}
