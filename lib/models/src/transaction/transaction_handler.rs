use crate::{
    transaction::TransactionLayerMsg,
    transport::{RequestMsg, ResponseMsg, TransportMsg},
    Error,
};
use common::{
    rsip::Response,
    tokio::sync::{mpsc, oneshot},
};

#[derive(Debug, Clone)]
pub struct TransactionHandler {
    inner: mpsc::Sender<TransactionLayerMsg>,
}

impl TransactionHandler {
    pub async fn process(&self, msg: TransportMsg) -> Result<(), Error> {
        Ok(self.inner.send(TransactionLayerMsg::Incoming(msg)).await?)
    }

    pub async fn reply(&self, msg: ResponseMsg) -> Result<(), Error> {
        Ok(self.inner.send(TransactionLayerMsg::Reply(msg)).await?)
    }

    pub async fn new_uac_invite(&self, msg: RequestMsg) -> Result<(), Error> {
        Ok(self
            .inner
            .send(TransactionLayerMsg::NewUacInvite(msg))
            .await?)
    }

    pub async fn new_uas_invite(
        &self,
        msg: RequestMsg,
        tu_response: Option<Response>,
    ) -> Result<(), Error> {
        Ok(self
            .inner
            .send(TransactionLayerMsg::NewUasInvite(msg, tu_response))
            .await?)
    }

    pub async fn has_transaction_for(&self, transaction_id: String) -> Result<bool, Error> {
        let (tx, rx) = oneshot::channel();

        self.inner
            .send(TransactionLayerMsg::HasTransaction(transaction_id, tx))
            .await?;
        Ok(rx.await?)
    }
}
