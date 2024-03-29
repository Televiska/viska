use crate::{
    transaction::{TransactionId, TransactionLayerMsg},
    Error,
};
use common::{
    rsip,
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

    pub async fn process(&self, msg: rsip::SipMessage) -> Result<(), Error> {
        Ok(self.tx.send(TransactionLayerMsg::Incoming(msg)).await?)
    }

    pub async fn reply(&self, msg: rsip::Response) -> Result<(), Error> {
        Ok(self.tx.send(TransactionLayerMsg::Reply(msg)).await?)
    }

    pub async fn transport_error(&self, msg: rsip::SipMessage, error: String) -> Result<(), Error> {
        Ok(self
            .tx
            .send(TransactionLayerMsg::TransportError(msg, error))
            .await?)
    }

    pub async fn new_uac_invite(&self, msg: rsip::Request) -> Result<(), Error> {
        Ok(self.tx.send(TransactionLayerMsg::NewUacInvite(msg)).await?)
    }

    pub async fn new_uas_invite(
        &self,
        msg: rsip::Request,
        tu_response: Option<rsip::Response>,
    ) -> Result<(), Error> {
        Ok(self
            .tx
            .send(TransactionLayerMsg::NewUasInvite(msg, tu_response))
            .await?)
    }

    pub async fn new_uac(&self, msg: rsip::Request) -> Result<(), Error> {
        Ok(self.tx.send(TransactionLayerMsg::NewUac(msg)).await?)
    }

    pub async fn new_uas(
        &self,
        msg: rsip::Request,
        tu_response: Option<rsip::Response>,
    ) -> Result<(), Error> {
        Ok(self
            .tx
            .send(TransactionLayerMsg::NewUas(msg, tu_response))
            .await?)
    }

    pub async fn has_transaction_for(&self, transaction_id: TransactionId) -> Result<bool, Error> {
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
