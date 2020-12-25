use crate::Error;
use crate::{core::CoreLayer, transaction::TransactionLayer, transport::TransportLayer};
use std::sync::Arc;

pub struct SipBuilder {
    pub manager: Arc<SipManager>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct SipManager {
    pub core: Arc<dyn CoreLayer>,
    pub transaction: Arc<dyn TransactionLayer>,
    pub transport: Arc<dyn TransportLayer>,
}

impl SipBuilder {
    pub fn new<C, Trx, T>() -> Result<Self, Error>
    where
        C: CoreLayer,
        Trx: TransactionLayer,
        T: TransportLayer,
    {
        Ok(Self {
            manager: Arc::new_cyclic(|me| SipManager {
                core: Arc::new(C::new(me.clone())),
                transaction: Arc::new(Trx::new(me.clone())),
                transport: Arc::new(T::new(me.clone()).expect("could not start transport")),
            }),
        })
    }

    pub async fn run(&self) {
        self.manager.transport.run().await;
    }
}
