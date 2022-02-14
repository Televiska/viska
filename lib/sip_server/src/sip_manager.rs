use crate::Error;
use crate::{tu::TuLayer, transaction::TransactionLayer, transport::TransportLayer};
use std::sync::Arc;

pub struct SipBuilder {
    pub manager: Arc<SipManager>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct SipManager {
    pub tu: Arc<dyn TuLayer>,
    pub transaction: Arc<dyn TransactionLayer>,
    pub transport: Arc<dyn TransportLayer>,
}

impl SipBuilder {
    pub fn new<C, Trx, T>() -> Result<Self, Error>
    where
        C: TuLayer,
        Trx: TransactionLayer,
        T: TransportLayer,
    {
        Ok(Self {
            manager: Arc::new_cyclic(|me| SipManager {
                tu: Arc::new(C::new(me.clone())),
                transaction: Arc::new(Trx::new(me.clone())),
                transport: Arc::new(T::new(me.clone()).expect("could not start transport")),
            }),
        })
    }

    pub async fn run(&self) {
        use common::futures::join;

        join!(self.manager.transaction.run(), self.manager.transport.run());
    }
}
