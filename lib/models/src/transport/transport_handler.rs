use crate::{
    transport::{TransportLayerMsg, TransportMsg, UdpTuple},
    Error,
};
use common::tokio::sync::mpsc::Sender;

#[derive(Debug, Clone)]
pub struct TransportHandler {
    pub tx: Sender<TransportLayerMsg>,
}

impl TransportHandler {
    pub fn new(tx: Sender<TransportLayerMsg>) -> Self {
        Self { tx }
    }

    pub async fn process(&self, msg: UdpTuple) -> Result<(), Error> {
        Ok(self.tx.send(TransportLayerMsg::Incoming(msg)).await?)
    }

    pub async fn send(&self, msg: TransportMsg) -> Result<(), Error> {
        Ok(self.tx.send(TransportLayerMsg::Outgoing(msg)).await?)
    }
}

impl From<Sender<TransportLayerMsg>> for TransportHandler {
    fn from(tx: Sender<TransportLayerMsg>) -> Self {
        Self { tx }
    }
}
