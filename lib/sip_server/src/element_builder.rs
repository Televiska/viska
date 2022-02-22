use crate::Error;
use crate::{transaction::Transaction, transport::Transport, tu::elements::UserAgent};
use models::Handlers;

pub struct ElementBuilder;

impl ElementBuilder {
    pub fn new() {
        let (tu_tx, tu_rx) = channel(10);
        let (transaction_tx, transaction_rx) = channel(10);
        let (transport_tx, transport_rx) = channel(10);

        let handlers: Handlers = (tu_tx, transaction_tx, transport_tx).into();

    }

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
