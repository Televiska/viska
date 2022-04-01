use crate::common::delay_for;
use common::async_trait::async_trait;
use models::transaction::TransactionId;
use sip_server::transaction::{
    sm::uac::TrxState as UacTrxState, sm::uas::TrxState as UasTrxState, sm::TrxStateSm,
};
use std::time::Duration;

#[async_trait]
pub trait TransactionUacExt {
    async fn is_uac_calling(&self, transaction_id: TransactionId) -> bool;
    async fn is_uac_proceeding(&self, transaction_id: TransactionId) -> bool;
    async fn is_uac_completed(&self, transaction_id: TransactionId) -> bool;
    async fn is_uac_accepted(&self, transaction_id: TransactionId) -> bool;
    async fn is_uac_terminated(&self, transaction_id: TransactionId) -> bool;
    async fn is_uac_errored(&self, transaction_id: TransactionId) -> bool;
}

#[async_trait]
impl TransactionUacExt for sip_server::Transaction {
    async fn is_uac_calling(&self, transaction_id: TransactionId) -> bool {
        delay_for(Duration::from_millis(1)).await;
        match self
            .inner
            .state
            .read()
            .await
            .get(&transaction_id)
            .expect("getting transaction from state")
        {
            TrxStateSm::Uac(sm) => {
                matches!(sm.lock().await.state, UacTrxState::Calling { .. })
            }
            _ => false,
        }
    }

    async fn is_uac_proceeding(&self, transaction_id: TransactionId) -> bool {
        delay_for(Duration::from_millis(1)).await;
        match self
            .inner
            .state
            .read()
            .await
            .get(&transaction_id)
            .expect("getting transaction from state")
        {
            TrxStateSm::Uac(sm) => {
                matches!(sm.lock().await.state, UacTrxState::Proceeding { .. })
            }
            _ => false,
        }
    }

    async fn is_uac_completed(&self, transaction_id: TransactionId) -> bool {
        delay_for(Duration::from_millis(1)).await;
        match self
            .inner
            .state
            .read()
            .await
            .get(&transaction_id)
            .expect("getting transaction from state")
        {
            TrxStateSm::Uac(sm) => {
                matches!(sm.lock().await.state, UacTrxState::Completed { .. })
            }
            _ => false,
        }
    }

    async fn is_uac_accepted(&self, transaction_id: TransactionId) -> bool {
        delay_for(Duration::from_millis(1)).await;
        match self
            .inner
            .state
            .read()
            .await
            .get(&transaction_id)
            .expect("getting transaction from state")
        {
            TrxStateSm::Uac(sm) => {
                matches!(sm.lock().await.state, UacTrxState::Accepted { .. })
            }
            _ => false,
        }
    }

    async fn is_uac_terminated(&self, transaction_id: TransactionId) -> bool {
        delay_for(Duration::from_millis(1)).await;
        match self
            .inner
            .state
            .read()
            .await
            .get(&transaction_id)
            .expect("getting transaction from state")
        {
            TrxStateSm::Uac(sm) => {
                matches!(sm.lock().await.state, UacTrxState::Terminated { .. })
            }
            _ => false,
        }
    }

    async fn is_uac_errored(&self, transaction_id: TransactionId) -> bool {
        delay_for(Duration::from_millis(1)).await;
        match self
            .inner
            .state
            .read()
            .await
            .get(&transaction_id)
            .expect("getting transaction from state")
        {
            TrxStateSm::Uac(sm) => {
                matches!(sm.lock().await.state, UacTrxState::Errored { .. })
            }
            _ => false,
        }
    }
}

#[async_trait]
pub trait TransactionUasExt {
    async fn is_uas_proceeding(&self, transaction_id: TransactionId) -> bool;
    async fn is_uas_completed(&self, transaction_id: TransactionId) -> bool;
    async fn is_uas_accepted(&self, transaction_id: TransactionId) -> bool;
    async fn is_uas_confirmed(&self, transaction_id: TransactionId) -> bool;
    async fn is_uas_terminated(&self, transaction_id: TransactionId) -> bool;
    async fn is_uas_errored(&self, transaction_id: TransactionId) -> bool;
}

#[async_trait]
impl TransactionUasExt for sip_server::Transaction {
    async fn is_uas_proceeding(&self, transaction_id: TransactionId) -> bool {
        delay_for(Duration::from_millis(1)).await;
        match self
            .inner
            .state
            .read()
            .await
            .get(&transaction_id)
            .expect("getting transaction from state")
        {
            TrxStateSm::Uas(sm) => {
                matches!(sm.lock().await.state, UasTrxState::Proceeding { .. })
            }
            _ => false,
        }
    }

    async fn is_uas_completed(&self, transaction_id: TransactionId) -> bool {
        delay_for(Duration::from_millis(1)).await;
        match self
            .inner
            .state
            .read()
            .await
            .get(&transaction_id)
            .expect("getting transaction from state")
        {
            TrxStateSm::Uas(sm) => {
                matches!(sm.lock().await.state, UasTrxState::Completed { .. })
            }
            _ => false,
        }
    }

    async fn is_uas_accepted(&self, transaction_id: TransactionId) -> bool {
        delay_for(Duration::from_millis(1)).await;
        match self
            .inner
            .state
            .read()
            .await
            .get(&transaction_id)
            .expect("getting transaction from state")
        {
            TrxStateSm::Uas(sm) => {
                matches!(sm.lock().await.state, UasTrxState::Accepted { .. })
            }
            _ => false,
        }
    }

    async fn is_uas_confirmed(&self, transaction_id: TransactionId) -> bool {
        delay_for(Duration::from_millis(1)).await;
        match self
            .inner
            .state
            .read()
            .await
            .get(&transaction_id)
            .expect("getting transaction from state")
        {
            TrxStateSm::Uas(sm) => {
                matches!(sm.lock().await.state, UasTrxState::Confirmed { .. })
            }
            _ => false,
        }
    }

    async fn is_uas_terminated(&self, transaction_id: TransactionId) -> bool {
        delay_for(Duration::from_millis(1)).await;
        match self
            .inner
            .state
            .read()
            .await
            .get(&transaction_id)
            .expect("getting transaction from state")
        {
            TrxStateSm::Uas(sm) => {
                matches!(sm.lock().await.state, UasTrxState::Terminated { .. })
            }
            _ => false,
        }
    }

    async fn is_uas_errored(&self, transaction_id: TransactionId) -> bool {
        delay_for(Duration::from_millis(1)).await;
        match self
            .inner
            .state
            .read()
            .await
            .get(&transaction_id)
            .expect("getting transaction from state")
        {
            TrxStateSm::Uas(sm) => {
                matches!(sm.lock().await.state, UasTrxState::Errored { .. })
            }
            _ => false,
        }
    }
}
