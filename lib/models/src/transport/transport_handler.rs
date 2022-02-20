use crate::{
    transport::{TransportLayerMsg, TransportMsg, UdpTuple},
    Error,
};
use common::tokio::sync::mpsc::Sender;

#[derive(Debug, Clone)]
pub struct TransportHandler {
    inner: Sender<TransportLayerMsg>,
}

impl TransportHandler {
    pub async fn process(&self, msg: UdpTuple) -> Result<(), Error> {
        Ok(self.inner.send(TransportLayerMsg::Incoming(msg)).await?)
    }

    pub async fn send(&self, msg: TransportMsg) -> Result<(), Error> {
        Ok(self.inner.send(TransportLayerMsg::Outgoing(msg)).await?)
    }
}
