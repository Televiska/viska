pub mod uac;
pub mod uas;

use crate::{error::TransactionError, Error};
use common::{
    rsip,
    tokio::{
        self,
        sync::{Mutex, RwLock},
    },
};
use models::{
    receivers::TrxReceiver,
    transaction::{TransactionHandler, TransactionLayerMsg},
    transport::{RequestMsg, ResponseMsg, TransportMsg},
    Handlers,
};
use std::collections::HashMap;
use std::{fmt::Debug, sync::Arc};

#[allow(dead_code)]
#[derive(Debug)]
pub struct Transaction {
    pub inner: Arc<Inner>,
}

#[derive(Debug)]
pub struct Inner {
    handlers: Handlers,
    pub uac_state: RwLock<HashMap<String, Mutex<uac::TrxStateMachine>>>,
    pub uas_state: RwLock<HashMap<String, Mutex<uas::TrxStateMachine>>>,
}

impl Transaction {
    pub fn new(handlers: Handlers, messages_rx: TrxReceiver) -> Result<Self, Error> {
        let me = Self {
            inner: Arc::new(Inner {
                handlers,
                uac_state: RwLock::new(Default::default()),
                uas_state: RwLock::new(Default::default()),
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
            if let Err(err) = self.receive(request.into()).await {
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
                .send(self.has_transaction(&transaction_id).await)
                .map_err(|e| Error::custom(format!("could not send respond: {}", e)))?,
        };

        Ok(())
    }

    //TODO: improve ergonomics
    async fn process_transport_error(
        &self,
        msg: TransportMsg,
        reason: String,
    ) -> Result<(), Error> {
        if msg.is_request() {
            match self
                .uas_state
                .read()
                .await
                .get(&msg.transaction_id()?.expect("transaction id"))
            {
                Some(transaction_machine) => {
                    let mut transaction_machine = transaction_machine.lock().await;
                    transaction_machine.transport_error(reason).await;
                    Ok(())
                }
                None => Err(Error::from(TransactionError::NotFound)),
            }
        } else {
            match self
                .uac_state
                .read()
                .await
                .get(&msg.transaction_id()?.expect("transaction id"))
            {
                Some(transaction_machine) => {
                    let mut transaction_machine = transaction_machine.lock().await;
                    transaction_machine.transport_error(reason).await;

                    Ok(())
                }
                None => Err(Error::from(TransactionError::NotFound)),
            }
        }
    }

    async fn new_uac_invite_transaction(&self, msg: RequestMsg) -> Result<(), Error> {
        let transaction_data = uac::TrxStateMachine::new(self.handlers.clone(), msg.clone())?;

        self.handlers.transport.send(msg.into()).await?;
        {
            let mut data = self.uac_state.write().await;
            data.insert(transaction_data.id.clone(), Mutex::new(transaction_data));
        }

        Ok(())
    }

    async fn new_uas_invite_transaction(
        &self,
        msg: RequestMsg,
        response: Option<rsip::Response>,
    ) -> Result<(), Error> {
        let transaction_data =
            uas::TrxStateMachine::new(self.handlers.clone(), msg.clone(), response)?;

        self.handlers.transport.send(msg.into()).await?;
        {
            let mut data = self.uas_state.write().await;
            data.insert(transaction_data.id.clone(), Mutex::new(transaction_data));
        }

        Ok(())
    }

    async fn process_tu_reply(&self, msg: ResponseMsg) -> Result<(), Error> {
        match self
            .uas_state
            .read()
            .await
            .get(&msg.transaction_id()?.expect("transaction_id"))
        {
            Some(transaction_machine) => {
                let mut transaction_machine = transaction_machine.lock().await;
                transaction_machine
                    .next(Some(msg.sip_response.into()))
                    .await?;
                Ok(())
            }
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

        match self.uas_state.read().await.get(&transaction_id) {
            Some(transaction_machine) => {
                let mut transaction_machine = transaction_machine.lock().await;
                transaction_machine
                    .next(Some(msg.sip_request.into()))
                    .await?;

                Ok(())
            }
            None => Err(Error::from(TransactionError::NotFound)),
        }
    }

    async fn process_incoming_response(&self, msg: ResponseMsg) -> Result<(), Error> {
        let transaction_id = msg.transaction_id()?.expect("transaction_id");

        match self.uac_state.read().await.get(&transaction_id) {
            Some(transaction_machine) => {
                let mut transaction_machine = transaction_machine.lock().await;
                transaction_machine.next(Some(msg.sip_response)).await;
                Ok(())
            }
            None => Err(Error::from(TransactionError::NotFound)),
        }
    }

    async fn has_transaction(&self, transaction_id: &str) -> bool {
        let uas_state = self.uas_state.read().await;
        let uac_state = self.uac_state.read().await;

        match (uas_state.get(transaction_id), uac_state.get(transaction_id)) {
            (Some(uas_trx), Some(uac_trx)) => {
                uas_trx.lock().await.is_active() || uac_trx.lock().await.is_active()
            }
            (Some(uas_trx), None) => uas_trx.lock().await.is_active(),
            (None, Some(uac_trx)) => uac_trx.lock().await.is_active(),
            (None, None) => false,
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
        {
            let uac_state = self.uac_state.read().await;
            for transaction_data in (*uac_state).values() {
                let mut transaction_data = transaction_data.lock().await;
                transaction_data.next(None).await;
            }
        }

        {
            let uas_state = self.uas_state.read().await;
            for transaction_data in (*uas_state).values() {
                let mut transaction_data = transaction_data.lock().await;
                transaction_data
                    .next(None)
                    .await
                    .expect("next on uas transaction");
            }
        }
    }
}
