pub mod uac;
pub mod uas;

use crate::{error::TransactionError, Error, SipManager};
use common::{
    async_trait::async_trait,
    rsip::prelude::*,
    tokio::{
        self,
        sync::{Mutex, RwLock},
    },
};
use models::transport::{RequestMsg, ResponseMsg, TransportMsg};
use std::collections::HashMap;
use std::{
    any::Any,
    fmt::Debug,
    sync::{Arc, Weak},
};

#[async_trait]
pub trait TransactionLayer: Send + Sync + Any + Debug {
    fn new(sip_manager: Weak<SipManager>) -> Self
    where
        Self: Sized;
    fn sip_manager(&self) -> Arc<SipManager>;
    async fn new_uac_invite_transaction(&self, msg: RequestMsg) -> Result<(), Error>;
    async fn new_uas_invite_transaction(
        &self,
        msg: RequestMsg,
        response: Option<rsip::Response>,
    ) -> Result<(), Error>;
    async fn has_transaction(&self, transaction_id: &str) -> bool;
    async fn process_incoming_message(&self, msg: TransportMsg);
    async fn send(&self, msg: ResponseMsg) -> Result<(), Error>;
    async fn run(&self);
    fn as_any(&self) -> &dyn Any;
}

#[allow(dead_code)]
pub struct Transaction {
    pub inner: Arc<Inner>,
}

pub struct Inner {
    sip_manager: Weak<SipManager>,
    pub uac_state: RwLock<HashMap<String, Mutex<uac::TrxStateMachine>>>,
    pub uas_state: RwLock<HashMap<String, Mutex<uas::TrxStateMachine>>>,
}

#[allow(dead_code)]
#[async_trait]
impl TransactionLayer for Transaction {
    fn new(sip_manager: Weak<SipManager>) -> Self {
        let inner = Arc::new(Inner {
            sip_manager,
            uac_state: RwLock::new(Default::default()),
            uas_state: RwLock::new(Default::default()),
        });

        Self { inner }
    }

    async fn has_transaction(&self, transaction_id: &str) -> bool {
        self.inner.has_transaction(transaction_id).await
    }

    async fn process_incoming_message(&self, msg: TransportMsg) {
        self.inner.process_incoming_message(msg).await
    }

    async fn new_uac_invite_transaction(&self, msg: RequestMsg) -> Result<(), Error> {
        self.inner.new_uac_invite_transaction(msg).await
    }

    async fn new_uas_invite_transaction(
        &self,
        msg: RequestMsg,
        response: Option<rsip::Response>,
    ) -> Result<(), Error> {
        self.inner.new_uas_invite_transaction(msg, response).await
    }

    async fn send(&self, msg: ResponseMsg) -> Result<(), Error> {
        self.inner.send(msg).await
    }

    async fn run(&self) {
        let inner = self.inner.clone();
        tokio::spawn(async move {
            inner.run().await;
        });
    }

    fn sip_manager(&self) -> Arc<SipManager> {
        self.inner.sip_manager()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Inner {
    //potential bug if state changes between this and process_incoming_message ?
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

    //returns error only if transaction is not found
    //TODO: maybe fix that in the future ?
    async fn process_incoming_message(&self, msg: TransportMsg) {
        match self.process_incoming(msg).await {
            Ok(_) => (),
            Err(error) => {
                common::log::warn!("error while processing incoming: {:?}", error);
            }
        }
    }

    async fn new_uac_invite_transaction(&self, msg: RequestMsg) -> Result<(), Error> {
        let transaction_data = uac::TrxStateMachine::new(self.sip_manager(), msg.clone())?;

        self.sip_manager().transport.send(msg.into()).await?;
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
            uas::TrxStateMachine::new(self.sip_manager(), msg.clone(), response)?;

        self.sip_manager().transport.send(msg.into()).await?;
        {
            let mut data = self.uas_state.write().await;
            data.insert(transaction_data.id.clone(), Mutex::new(transaction_data));
        }

        Ok(())
    }

    async fn send(&self, msg: ResponseMsg) -> Result<(), Error> {
        match self.uas_state.read().await.get(&msg.transaction_id()?) {
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

    async fn run(&self) {
        use tokio::time;

        let mut ticker = time::interval(time::Duration::from_millis(100));
        loop {
            ticker.tick().await;

            self.check_transactions().await
        }
    }

    fn sip_manager(&self) -> Arc<SipManager> {
        self.sip_manager.upgrade().expect("sip manager is missing!")
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
        let transaction_id = msg
            .sip_request
            .transaction_id()
            .map_err(|_| Error::from(TransactionError::NotFound))?;

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
        let transaction_id = msg
            .sip_response
            .transaction_id()
            .map_err(|_| Error::from(TransactionError::NotFound))?;

        match self.uac_state.read().await.get(&transaction_id) {
            Some(transaction_machine) => {
                let mut transaction_machine = transaction_machine.lock().await;
                transaction_machine.next(Some(msg.sip_response)).await;
                Ok(())
            }
            None => Err(Error::from(TransactionError::NotFound)),
        }
    }
}

impl std::fmt::Debug for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Transaction")
            .field("state", &self.inner.uac_state)
            .finish()
    }
}
