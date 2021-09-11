mod processor;
mod register;

pub use processor::UacProcessor;
pub use register::{RegisterConfig, Register};

use crate::CoreLayer;
use common::{async_trait::async_trait, tokio, rsip, CONFIG};
use std::{
    any::Any,
    sync::{Arc, Weak},
};

use crate::SipManager;
use models::transport::TransportMsg;

pub struct UserAgent {
    sip_manager: Weak<SipManager>,
    processor: Arc<UacProcessor>,
    register: Arc<Register>,
}

#[async_trait]
impl CoreLayer for UserAgent {
    fn new(sip_manager: Weak<SipManager>) -> Self {
        Self {
            sip_manager: sip_manager.clone(),
            processor: Arc::new(UacProcessor::new(sip_manager.clone())),
            register: Arc::new(Register::new(
                sip_manager,
                RegisterConfig {
                    auth: rsip::Auth {
                        user: "viska".into(),
                        password: None,
                    },
                    upstream: rsip::Host::from("192.168.0.30").into(),
                    downstream: CONFIG.default_addr().into(),
                    scheme: rsip::Scheme::Sip,
                    expiration: None,
                    refresh_interval: None
                },
            )),
        }
    }

    async fn process_incoming_message(&self, msg: TransportMsg) {
        let processor = self.processor.clone();
        tokio::spawn(async move {
            match processor.process_incoming_message(msg).await {
                Ok(()) => (),
                Err(err) => common::log::warn!("failed to process message: {:?}", err),
            }
        });
    }

    async fn send(&self, msg: TransportMsg) {
        match self.sip_manager().transport.send(msg).await {
            Ok(_) => (),
            Err(err) => common::log::error!("failed to send message: {:?}", err),
        }
    }

    async fn run(&self) {
        self.register.send_registration_request().await;
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl UserAgent {
    fn sip_manager(&self) -> Arc<SipManager> {
        self.sip_manager.upgrade().expect("sip manager is missing!")
    }
}

impl std::fmt::Debug for UserAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Core")
            .field("processor", &self.processor)
            .finish()
    }
}
