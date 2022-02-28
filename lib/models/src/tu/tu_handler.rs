use crate::{transport::TransportMsg, tu::TuLayerMsg, Error};
use common::tokio::sync::mpsc::Sender;

#[derive(Debug, Clone)]
pub struct TuHandler {
    pub tx: Sender<TuLayerMsg>,
}

impl TuHandler {
    pub fn new(tx: Sender<TuLayerMsg>) -> Self {
        Self { tx }
    }

    pub async fn process(&self, msg: TransportMsg) -> Result<(), Error> {
        Ok(self.tx.send(TuLayerMsg::Incoming(msg)).await?)
    }

    pub async fn transport_error(&self, msg: TransportMsg, error: String) -> Result<(), Error> {
        Ok(self.tx.send(TuLayerMsg::TransportError(msg, error)).await?)
    }
}

impl From<Sender<TuLayerMsg>> for TuHandler {
    fn from(tx: Sender<TuLayerMsg>) -> Self {
        Self { tx }
    }
}
