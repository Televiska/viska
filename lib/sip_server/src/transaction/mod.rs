pub mod sm;

use crate::{error::TransactionError, Error};
use common::{
    rsip,
    tokio::{self, sync::RwLock},
};
use models::{
    receivers::TrxReceiver,
    transaction::{TransactionHandler, TransactionLayerMsg},
    transport::{RequestMsg, ResponseMsg, TransportMsg},
    Handlers,
};
use sm::TrxStateSm;
use std::{collections::HashMap, fmt::Debug, sync::Arc};

#[allow(dead_code)]
#[derive(Debug)]
pub struct Transaction {
    pub inner: Arc<Inner>,
}

#[derive(Debug)]
pub struct Inner {
    handlers: Handlers,
    pub state: RwLock<HashMap<String, TrxStateSm>>,
}

impl Transaction {
    pub fn new(handlers: Handlers, messages_rx: TrxReceiver) -> Result<Self, Error> {
        let me = Self {
            inner: Arc::new(Inner {
                handlers,
                state: RwLock::new(Default::default()),
            }),
        };

        me.run(messages_rx);

        Ok(me)
    }

    pub fn handler(&self) -> TransactionHandler {
        self.inner.handlers.transaction.clone()
    }

    fn run(&self, messages: TrxReceiver) {
        let inner = self.inner.clone();
        tokio::spawn(async move { inner.run(messages).await });
        let inner_trx = self.inner.clone();
        tokio::spawn(async move { inner_trx.run_transactions().await });
    }
}

impl Inner {
    async fn run(&self, mut messages: TrxReceiver) {
        while let Some(request) = messages.recv().await {
            if let Err(err) = self.receive(request).await {
                common::log::error!("Error handling transaction layer message: {}", err)
            }
        }
    }

    //TODO: here we don't spawn, could lead to deadlocks
    async fn receive(&self, msg: TransactionLayerMsg) -> Result<(), Error> {
        match msg {
            TransactionLayerMsg::NewUacInvite(msg) => self.new_uac_invite_transaction(msg).await?,
            TransactionLayerMsg::NewUasInvite(msg, response) => {
                self.new_uas_invite_transaction(msg, response).await?
            }
            TransactionLayerMsg::Reply(msg) => self.process_tu_reply(msg).await?,
            TransactionLayerMsg::Incoming(msg) => self.process_incoming(msg).await?,
            TransactionLayerMsg::TransportError(msg, error) => {
                self.process_transport_error(msg, error).await?
            }
            TransactionLayerMsg::HasTransaction(transaction_id, tx) => tx
                .send(self.exists(transaction_id).await)
                .map_err(|e| Error::custom(format!("could not send respond: {}", e)))?,
        };

        Ok(())
    }

    pub async fn exists(&self, transaction_id: String) -> bool {
        let state = self.state.read().await;

        state.get(&transaction_id).is_some()
    }

    async fn process_transport_error(
        &self,
        msg: TransportMsg,
        reason: String,
    ) -> Result<(), Error> {
        let transaction_id = msg.transaction_id()?.expect("transaction_id");
        if let Some(sm) = self.state.read().await.get(&transaction_id) {
            sm.transport_error(reason).await;

            return Ok(());
        }

        Err(Error::from(TransactionError::NotFound))
    }

    async fn new_uac_invite_transaction(&self, msg: RequestMsg) -> Result<(), Error> {
        self.handlers.transport.send(msg.clone().into()).await?;
        let transaction_data = sm::uac::TrxStateMachine::new(self.handlers.clone(), msg.clone())?;
        {
            let mut data = self.state.write().await;
            data.insert(transaction_data.id.clone(), transaction_data.into());
        }
        Ok(())
    }

    async fn new_uas_invite_transaction(
        &self,
        msg: RequestMsg,
        response: Option<rsip::Response>,
    ) -> Result<(), Error> {
        self.handlers.transport.send(msg.clone().into()).await?;
        let transaction_data =
            sm::uas::TrxStateMachine::new(self.handlers.clone(), msg.clone(), response)?;

        {
            let mut data = self.state.write().await;
            data.insert(transaction_data.id.clone(), transaction_data.into());
        }

        Ok(())
    }

    async fn process_tu_reply(&self, msg: ResponseMsg) -> Result<(), Error> {
        let transaction_id = msg.transaction_id()?.expect("transaction_id");

        match self.state.read().await.get(&transaction_id) {
            Some(sm) => Ok(sm.uas_process_tu_reply(msg).await?),
            None => Err(Error::from(TransactionError::NotFound)),
        }
    }

    async fn process_incoming(&self, msg: TransportMsg) -> Result<(), Error> {
        let sip_message = msg.sip_message;

        match sip_message {
            rsip::SipMessage::Request(request) => {
                self.process_incoming_request(RequestMsg::new(request, msg.peer, msg.transport))
                    .await?
            }
            rsip::SipMessage::Response(response) => {
                self.process_incoming_response(ResponseMsg::new(response, msg.peer, msg.transport))
                    .await?
            }
        }

        Ok(())
    }

    async fn process_incoming_request(&self, msg: RequestMsg) -> Result<(), Error> {
        let transaction_id = msg.transaction_id()?.expect("transaction_id");

        match self.state.read().await.get(&transaction_id) {
            Some(sm) => Ok(sm.uas_process_request(msg).await?),
            None => Err(Error::from(TransactionError::NotFound)),
        }
    }

    async fn process_incoming_response(&self, msg: ResponseMsg) -> Result<(), Error> {
        let transaction_id = msg.transaction_id()?.expect("transaction_id");

        match self.state.read().await.get(&transaction_id) {
            Some(sm) => Ok(sm.uac_process_response(msg).await?),
            None => Err(Error::from(TransactionError::NotFound)),
        }
    }

    async fn run_transactions(&self) {
        use tokio::time;

        let mut ticker = time::interval(time::Duration::from_millis(100));
        loop {
            ticker.tick().await;

            self.check_transactions().await
        }
    }

    async fn check_transactions(&self) {
        let state = self.state.read().await;
        for transaction_data in (*state).values() {
            match transaction_data {
                TrxStateSm::Uac(sm) => sm.lock().await.next(None).await,
                TrxStateSm::Uas(sm) => sm.lock().await.next(None).await,
            };
        }
    }
}
