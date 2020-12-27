pub mod uac;
pub mod uas;

use crate::{error, Error, SipManager};
use common::async_trait::async_trait;
use models::{
    transport::{RequestMsg, ResponseMsg, TransportMsg},
    SipMessageExt,
};
use rsip::SipMessage;
use std::collections::HashMap;
use std::{
    any::Any,
    fmt::Debug,
    sync::{Arc, Weak},
};
use tokio::sync::{Mutex, RwLock};

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
    async fn process_incoming_message(&self, msg: TransportMsg);
    async fn send(&self, msg: ResponseMsg) -> Result<(), Error>;
    async fn run(&self) -> Result<(), Error>;
    fn as_any(&self) -> &dyn Any;
}

#[allow(dead_code)]
pub struct Transaction {
    sip_manager: Weak<SipManager>,
    pub uac_state: RwLock<HashMap<String, Mutex<uac::TrxStateMachine>>>,
    pub uas_state: RwLock<HashMap<String, Mutex<uas::TrxStateMachine>>>,
}

#[allow(dead_code)]
#[async_trait]
impl TransactionLayer for Transaction {
    fn new(sip_manager: Weak<SipManager>) -> Self {
        Self {
            sip_manager,
            uac_state: RwLock::new(Default::default()),
            uas_state: RwLock::new(Default::default()),
        }
    }

    fn sip_manager(&self) -> Arc<SipManager> {
        self.sip_manager.upgrade().expect("sip manager is missing!")
    }

    async fn process_incoming_message(&self, msg: TransportMsg) {
        match self.process_incoming(msg).await {
            Ok(_) => (),
            Err(error) => common::log::warn!("error while processing incoming: {:?}", error),
        };
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
            None => Err(Error::from(error::Transaction::NotFound)),
        }
    }

    async fn run(&self) -> Result<(), Error> {
        use tokio::time;

        let mut ticker = time::interval(time::Duration::from_millis(100));
        loop {
            ticker.tick().await;

            self.check_transactions().await
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Transaction {
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
            SipMessage::Request(request) => {
                self.process_incoming_request(RequestMsg::new(request, msg.peer, msg.transport))
                    .await?
            }
            SipMessage::Response(response) => {
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
            .map_err(|_| Error::from(error::Transaction::NotFound))?;

        match self.uas_state.read().await.get(&transaction_id) {
            Some(transaction_machine) => {
                let mut transaction_machine = transaction_machine.lock().await;
                transaction_machine
                    .next(Some(msg.sip_request.into()))
                    .await?;

                Ok(())
            }
            None => Err(Error::from(error::Transaction::NotFound)),
        }
    }

    async fn process_incoming_response(&self, msg: ResponseMsg) -> Result<(), Error> {
        let transaction_id = msg
            .sip_response
            .transaction_id()
            .map_err(|_| Error::from(error::Transaction::NotFound))?;

        match self.uac_state.read().await.get(&transaction_id) {
            Some(transaction_machine) => {
                let mut transaction_machine = transaction_machine.lock().await;
                transaction_machine.next(Some(msg.sip_response)).await;
                Ok(())
            }
            None => Err(Error::from(error::Transaction::NotFound)),
        }
    }
}

impl std::fmt::Debug for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Transaction")
            .field("state", &self.uac_state)
            .finish()
    }
}
