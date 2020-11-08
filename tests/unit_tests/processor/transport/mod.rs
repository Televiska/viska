pub mod processor;

use ::processor::{CoreLayer, SipBuilder, SipManager, Transaction, Transport};
use common::async_trait::async_trait;
use models::{server::UdpTuple, transport::TransportMsg};
use std::any::Any;
use std::sync::{Arc, Weak};
use tokio::sync::Mutex;

pub struct CoreSnitch {
    sip_manager: Weak<SipManager>,
    pub messages: Mutex<Vec<TransportMsg>>,
}

#[async_trait]
impl CoreLayer for CoreSnitch {
    fn new(sip_manager: Weak<SipManager>) -> Self {
        Self {
            sip_manager: sip_manager.clone(),
            messages: Mutex::new(vec![]),
        }
    }

    async fn process_incoming_message(&self, msg: TransportMsg) {
        let mut messages = self.messages.lock().await;
        messages.push(msg);
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

async fn setup() -> Arc<SipManager> {
    SipBuilder::new::<CoreSnitch, Transaction, Transport>()
        .expect("sip manager failed")
        .manager
}
