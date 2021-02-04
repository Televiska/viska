mod capabilities;
mod processor;
mod registrar;

pub use capabilities::Capabilities;
pub use processor::Processor;
pub use registrar::Registrar;

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
    async fn run(&self);
    fn as_any(&self) -> &dyn Any;
}

pub struct Core {
    inner: Arc<Inner>,
}

#[async_trait]
impl CoreLayer for Core {
    fn new(sip_manager: Weak<SipManager>) -> Self {
        let inner = Arc::new(Inner {
            sip_manager: sip_manager.clone(),
            processor: Arc::new(Processor::new(sip_manager)),
        });
        Self { inner }
    }

    async fn process_incoming_message(&self, msg: TransportMsg) {
        self.inner.process_incoming_message(msg).await
    }

    async fn send(&self, msg: TransportMsg) {
        self.inner.send(msg).await
    }

    async fn run(&self) {
        let inner = self.inner.clone();
        tokio::spawn(async move {
            inner.run().await;
        });
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

struct Inner {
    sip_manager: Weak<SipManager>,
    processor: Arc<Processor>,
}

impl Inner {
    //TODO: remove expect and log instead
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

    async fn run(&self) {}
}

impl std::fmt::Debug for Core {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Core")
            .field("processor", &self.inner.processor)
            .finish()
    }
}
