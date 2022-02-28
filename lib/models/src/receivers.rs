use crate::{transaction::TransactionLayerMsg, transport::TransportLayerMsg, tu::TuLayerMsg};
use common::tokio::sync::mpsc::Receiver;

pub type TuReceiver = Receiver<TuLayerMsg>;
pub type TrxReceiver = Receiver<TransactionLayerMsg>;
pub type TrReceiver = Receiver<TransportLayerMsg>;

pub struct Receivers {
    pub tu: TuReceiver,
    pub transaction: TrxReceiver,
    pub transport: TrReceiver,
}

impl From<(TuReceiver, TrxReceiver, TrReceiver)> for Receivers {
    fn from(from: (TuReceiver, TrxReceiver, TrReceiver)) -> Self {
        Self {
            tu: from.0,
            transaction: from.1,
            transport: from.2,
        }
    }
}
