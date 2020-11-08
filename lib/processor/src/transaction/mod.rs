use crate::SipManager;
use common::async_trait::async_trait;
use std::any::Any;
use std::sync::{Arc, Weak};

#[async_trait]
pub trait TransactionLayer: Send + Sync + Any {
    fn new(sip_manager: Weak<SipManager>) -> Self
    where
        Self: Sized;
    fn sip_manager(&self) -> Arc<SipManager>;
    fn as_any(&self) -> &dyn Any;
}

#[allow(dead_code)]
pub struct Transaction {
    sip_manager: Weak<SipManager>,
}

#[allow(dead_code)]
#[async_trait]
impl TransactionLayer for Transaction {
    fn new(sip_manager: Weak<SipManager>) -> Self {
        Self { sip_manager }
    }

    fn sip_manager(&self) -> Arc<SipManager> {
        self.sip_manager.upgrade().expect("sip manager is missing!")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
