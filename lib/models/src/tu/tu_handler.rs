use crate::{Error, transport::TransportMsg, tu::TuLayerMsg};
use common::tokio::sync::mpsc::Sender;

#[derive(Debug, Clone)]
pub struct TuHandler {
    inner: Sender<TuLayerMsg>,
}

impl TuHandler {
    pub async fn process(&self, msg: TransportMsg) -> Result<(), Error> {
        Ok(self.inner.send(TuLayerMsg::Incoming(msg)).await?)
    }
}
