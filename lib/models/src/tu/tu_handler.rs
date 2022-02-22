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
}

impl From<Sender<TuLayerMsg>> for TuHandler {
    fn from(tx: Sender<TuLayerMsg>) -> Self {
        Self { tx }
    }
}
