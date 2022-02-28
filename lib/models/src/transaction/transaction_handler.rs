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
    pub tx: mpsc::Sender<TransactionLayerMsg>,
}

impl TransactionHandler {
    pub fn new(tx: mpsc::Sender<TransactionLayerMsg>) -> Self {
        Self { tx }
    }

    pub async fn process(&self, msg: TransportMsg) -> Result<(), Error> {
        Ok(self.tx.send(TransactionLayerMsg::Incoming(msg)).await?)
    }

    pub async fn reply(&self, msg: ResponseMsg) -> Result<(), Error> {
        Ok(self.tx.send(TransactionLayerMsg::Reply(msg)).await?)
    }

    pub async fn transport_error(&self, msg: TransportMsg, error: String) -> Result<(), Error> {
        Ok(self
            .tx
            .send(TransactionLayerMsg::TransportError(msg, error))
            .await?)
    }

    pub async fn new_uac_invite(&self, msg: RequestMsg) -> Result<(), Error> {
        Ok(self.tx.send(TransactionLayerMsg::NewUacInvite(msg)).await?)
    }

    pub async fn new_uas_invite(
        &self,
        msg: RequestMsg,
        tu_response: Option<Response>,
    ) -> Result<(), Error> {
        Ok(self
            .tx
            .send(TransactionLayerMsg::NewUasInvite(msg, tu_response))
            .await?)
    }

    pub async fn has_transaction_for(&self, transaction_id: String) -> Result<bool, Error> {
        let (tx, rx) = oneshot::channel();

        self.tx
            .send(TransactionLayerMsg::HasTransaction(transaction_id, tx))
            .await?;
        Ok(rx.await?)
    }
}

impl From<mpsc::Sender<TransactionLayerMsg>> for TransactionHandler {
    fn from(tx: mpsc::Sender<TransactionLayerMsg>) -> Self {
        Self { tx }
    }
}
