use common::async_trait::async_trait;
use sip_server::{
    transaction::{uac::TrxState as UacTrxState, uas::TrxState as UasTrxState},
    Transaction,
};

#[async_trait]
pub trait TransactionUacExt {
    async fn is_uac_calling(&self, transaction_id: String) -> bool;
    async fn is_uac_proceeding(&self, transaction_id: String) -> bool;
    async fn is_uac_completed(&self, transaction_id: String) -> bool;
    async fn is_uac_accepted(&self, transaction_id: String) -> bool;
    async fn is_uac_terminated(&self, transaction_id: String) -> bool;
    async fn is_uac_errored(&self, transaction_id: String) -> bool;
}

#[async_trait]
impl TransactionUacExt for sip_server::Transaction {
    async fn is_uac_calling(&self, transaction_id: String) -> bool {
        let state_reader = self.inner.uac_state.read().await;
        let transaction_data = state_reader
            .get(&transaction_id)
            .expect("getting transaction from state")
            .lock()
            .await;

        matches!(transaction_data.state, UacTrxState::Calling { .. })
    }

    async fn is_uac_proceeding(&self, transaction_id: String) -> bool {
        let state_reader = self.inner.uac_state.read().await;
        let transaction_data = state_reader
            .get(&transaction_id)
            .expect("getting transaction from state")
            .lock()
            .await;

        matches!(transaction_data.state, UacTrxState::Proceeding { .. })
    }

    async fn is_uac_completed(&self, transaction_id: String) -> bool {
        let state_reader = self.inner.uac_state.read().await;
        let transaction_data = state_reader
            .get(&transaction_id)
            .expect("getting transaction from state")
            .lock()
            .await;

        matches!(transaction_data.state, UacTrxState::Completed { .. })
    }

    async fn is_uac_accepted(&self, transaction_id: String) -> bool {
        let state_reader = self.inner.uac_state.read().await;
        let transaction_data = state_reader
            .get(&transaction_id)
            .expect("getting transaction from state")
            .lock()
            .await;

        matches!(transaction_data.state, UacTrxState::Accepted { .. })
    }

    async fn is_uac_terminated(&self, transaction_id: String) -> bool {
        let state_reader = self.inner.uac_state.read().await;
        let transaction_data = state_reader
            .get(&transaction_id)
            .expect("getting transaction from state")
            .lock()
            .await;

        matches!(transaction_data.state, UacTrxState::Terminated { .. })
    }

    async fn is_uac_errored(&self, transaction_id: String) -> bool {
        let state_reader = self.inner.uac_state.read().await;
        let transaction_data = state_reader
            .get(&transaction_id)
            .expect("getting transaction from state")
            .lock()
            .await;

        matches!(transaction_data.state, UacTrxState::Errored { .. })
    }
}

#[async_trait]
pub trait TransactionUasExt {
    async fn is_uas_proceeding(&self, transaction_id: String) -> bool;
    async fn is_uas_completed(&self, transaction_id: String) -> bool;
    async fn is_uas_accepted(&self, transaction_id: String) -> bool;
    async fn is_uas_confirmed(&self, transaction_id: String) -> bool;
    async fn is_uas_terminated(&self, transaction_id: String) -> bool;
    async fn is_uas_errored(&self, transaction_id: String) -> bool;
}

#[async_trait]
impl TransactionUasExt for sip_server::Transaction {
    async fn is_uas_proceeding(&self, transaction_id: String) -> bool {
        let state_reader = self.inner.uas_state.read().await;
        let transaction_data = state_reader
            .get(&transaction_id)
            .expect("getting transaction from state")
            .lock()
            .await;

        matches!(transaction_data.state, UasTrxState::Proceeding { .. })
    }

    async fn is_uas_completed(&self, transaction_id: String) -> bool {
        let state_reader = self.inner.uas_state.read().await;
        let transaction_data = state_reader
            .get(&transaction_id)
            .expect("getting transaction from state")
            .lock()
            .await;

        matches!(transaction_data.state, UasTrxState::Completed { .. })
    }

    async fn is_uas_accepted(&self, transaction_id: String) -> bool {
        let state_reader = self.inner.uas_state.read().await;
        let transaction_data = state_reader
            .get(&transaction_id)
            .expect("getting transaction from state")
            .lock()
            .await;

        matches!(transaction_data.state, UasTrxState::Accepted { .. })
    }

    async fn is_uas_confirmed(&self, transaction_id: String) -> bool {
        let state_reader = self.inner.uas_state.read().await;
        let transaction_data = state_reader
            .get(&transaction_id)
            .expect("getting transaction from state")
            .lock()
            .await;

        matches!(transaction_data.state, UasTrxState::Confirmed { .. })
    }

    async fn is_uas_terminated(&self, transaction_id: String) -> bool {
        let state_reader = self.inner.uas_state.read().await;
        let transaction_data = state_reader
            .get(&transaction_id)
            .expect("getting transaction from state")
            .lock()
            .await;

        matches!(transaction_data.state, UasTrxState::Terminated { .. })
    }

    async fn is_uas_errored(&self, transaction_id: String) -> bool {
        let state_reader = self.inner.uas_state.read().await;
        let transaction_data = state_reader
            .get(&transaction_id)
            .expect("getting transaction from state")
            .lock()
            .await;

        matches!(transaction_data.state, UasTrxState::Errored { .. })
    }
}
