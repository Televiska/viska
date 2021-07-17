use super::CoreLayer;
use crate::core::CoreProcessor;
use common::{async_trait::async_trait, rsip, tokio};
use std::{
    any::Any,
    sync::{Arc, Weak},
};

use crate::{Error, SipManager};
use models::transport::TransportMsg;

//TODO: rename this to something else like ProxyCore etc
pub struct Core<P: CoreProcessor> {
    inner: Arc<Inner<P>>,
}

#[async_trait]
impl<P: CoreProcessor> CoreLayer for Core<P> {
    fn new(sip_manager: Weak<SipManager>) -> Self {
        let inner = Arc::new(Inner {
            sip_manager: sip_manager.clone(),
            processor: Arc::new(P::new(sip_manager)),
        });
        Self { inner }
    }

    async fn process_incoming_message(&self, msg: TransportMsg) {
        self.inner.process_incoming_message(msg).await
    }

    async fn send(&self, request: rsip::Request) -> Result<(), Error> {
        Ok(self.inner.send(request).await?)
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

struct Inner<P: CoreProcessor> {
    sip_manager: Weak<SipManager>,
    processor: Arc<P>,
}

impl<P: CoreProcessor> Inner<P> {
    //TODO: the spawning here should go inside the processor
    async fn process_incoming_message(&self, msg: TransportMsg) {
        let processor = self.processor.clone();
        tokio::spawn(async move {
            match processor.process_incoming_message(msg).await {
                Ok(()) => (),
                Err(err) => common::log::warn!("failed to process message: {:?}", err),
            }
        });
    }

    async fn send(&self, request: rsip::Request) -> Result<(), Error> {
        let processor = self.processor.clone();
        tokio::spawn(async move {
            match processor.send(request).await {
                Ok(()) => (),
                Err(err) => common::log::warn!("processor failed to send message: {:?}", err),
            }
        });

        Ok(())
    }

    fn sip_manager(&self) -> Arc<SipManager> {
        self.sip_manager.upgrade().expect("sip manager is missing!")
    }

    async fn run(&self) {}
}

impl<P: CoreProcessor> std::fmt::Debug for Core<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Core")
            .field("processor", &self.inner.processor)
            .finish()
    }
}
