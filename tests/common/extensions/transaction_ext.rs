use crate::common::delay_for;
use common::async_trait::async_trait;
use models::transaction::TransactionId;
use sip_server::transaction::sm::{uac, uac_invite, uas_invite, TrxStateSm};
use std::time::Duration;

#[async_trait]
pub trait TransactionUacInviteExt {
    async fn is_calling(&self, transaction_id: TransactionId) -> bool;
    async fn is_proceeding(&self, transaction_id: TransactionId) -> bool;
    async fn is_completed(&self, transaction_id: TransactionId) -> bool;
    async fn is_accepted(&self, transaction_id: TransactionId) -> bool;
    async fn is_terminated(&self, transaction_id: TransactionId) -> bool;
    async fn is_timedout(&self, transaction_id: TransactionId) -> bool;
    async fn is_errored(&self, transaction_id: TransactionId) -> bool;
}

#[async_trait]
impl TransactionUacInviteExt for sip_server::Transaction {
    async fn is_calling(&self, transaction_id: TransactionId) -> bool {
        delay_for(Duration::from_millis(1)).await;
        match self
            .inner
            .state
            .read()
            .await
            .get(&transaction_id)
            .expect("getting transaction from state")
        {
            TrxStateSm::UacInvite(sm) => {
                matches!(sm.lock().await.state, uac_invite::TrxState::Calling { .. })
            }
            _ => false,
        }
    }

    async fn is_proceeding(&self, transaction_id: TransactionId) -> bool {
        delay_for(Duration::from_millis(1)).await;
        match self
            .inner
            .state
            .read()
            .await
            .get(&transaction_id)
            .expect("getting transaction from state")
        {
            TrxStateSm::UacInvite(sm) => {
                matches!(
                    sm.lock().await.state,
                    uac_invite::TrxState::Proceeding { .. }
                )
            }
            _ => false,
        }
    }

    async fn is_completed(&self, transaction_id: TransactionId) -> bool {
        delay_for(Duration::from_millis(1)).await;
        match self
            .inner
            .state
            .read()
            .await
            .get(&transaction_id)
            .expect("getting transaction from state")
        {
            TrxStateSm::UacInvite(sm) => {
                matches!(
                    sm.lock().await.state,
                    uac_invite::TrxState::Completed { .. }
                )
            }
            _ => false,
        }
    }

    async fn is_accepted(&self, transaction_id: TransactionId) -> bool {
        delay_for(Duration::from_millis(1)).await;
        match self
            .inner
            .state
            .read()
            .await
            .get(&transaction_id)
            .expect("getting transaction from state")
        {
            TrxStateSm::UacInvite(sm) => {
                matches!(sm.lock().await.state, uac_invite::TrxState::Accepted { .. })
            }
            _ => false,
        }
    }

    async fn is_terminated(&self, transaction_id: TransactionId) -> bool {
        delay_for(Duration::from_millis(1)).await;
        match self
            .inner
            .state
            .read()
            .await
            .get(&transaction_id)
            .expect("getting transaction from state")
        {
            TrxStateSm::UacInvite(sm) => {
                matches!(
                    sm.lock().await.state,
                    uac_invite::TrxState::Terminated(uac_invite::Terminated::Expected { .. })
                )
            }
            _ => false,
        }
    }

    async fn is_timedout(&self, transaction_id: TransactionId) -> bool {
        delay_for(Duration::from_millis(1)).await;
        match self
            .inner
            .state
            .read()
            .await
            .get(&transaction_id)
            .expect("getting transaction from state")
        {
            TrxStateSm::UacInvite(sm) => {
                matches!(
                    sm.lock().await.state,
                    uac_invite::TrxState::Terminated(uac_invite::Terminated::TimedOut { .. })
                )
            }
            _ => false,
        }
    }

    async fn is_errored(&self, transaction_id: TransactionId) -> bool {
        delay_for(Duration::from_millis(1)).await;
        match self
            .inner
            .state
            .read()
            .await
            .get(&transaction_id)
            .expect("getting transaction from state")
        {
            TrxStateSm::UacInvite(sm) => {
                matches!(
                    sm.lock().await.state,
                    uac_invite::TrxState::Terminated(uac_invite::Terminated::Errored { .. })
                )
            }
            _ => false,
        }
    }
}

#[async_trait]
pub trait TransactionUacExt {
    async fn is_trying(&self, transaction_id: TransactionId) -> bool;
    async fn is_proceeding(&self, transaction_id: TransactionId) -> bool;
    async fn is_completed(&self, transaction_id: TransactionId) -> bool;
    async fn is_terminated(&self, transaction_id: TransactionId) -> bool;
    async fn is_timedout(&self, transaction_id: TransactionId) -> bool;
    async fn is_errored(&self, transaction_id: TransactionId) -> bool;
}

#[async_trait]
impl TransactionUacExt for sip_server::Transaction {
    async fn is_trying(&self, transaction_id: TransactionId) -> bool {
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
                matches!(sm.lock().await.state, uac::TrxState::Trying { .. })
            }
            _ => false,
        }
    }

    async fn is_proceeding(&self, transaction_id: TransactionId) -> bool {
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
                matches!(sm.lock().await.state, uac::TrxState::Proceeding { .. })
            }
            _ => false,
        }
    }

    async fn is_completed(&self, transaction_id: TransactionId) -> bool {
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
                matches!(sm.lock().await.state, uac::TrxState::Completed { .. })
            }
            _ => false,
        }
    }

    async fn is_terminated(&self, transaction_id: TransactionId) -> bool {
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
                matches!(
                    sm.lock().await.state,
                    uac::TrxState::Terminated(uac::Terminated::Expected { .. })
                )
            }
            _ => false,
        }
    }

    async fn is_timedout(&self, transaction_id: TransactionId) -> bool {
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
                matches!(
                    sm.lock().await.state,
                    uac::TrxState::Terminated(uac::Terminated::TimedOut { .. })
                )
            }
            _ => false,
        }
    }

    async fn is_errored(&self, transaction_id: TransactionId) -> bool {
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
                matches!(
                    sm.lock().await.state,
                    uac::TrxState::Terminated(uac::Terminated::Errored { .. })
                )
            }
            _ => false,
        }
    }
}

#[async_trait]
pub trait TransactionUasInviteExt {
    async fn is_proceeding(&self, transaction_id: TransactionId) -> bool;
    async fn is_completed(&self, transaction_id: TransactionId) -> bool;
    async fn is_accepted(&self, transaction_id: TransactionId) -> bool;
    async fn is_confirmed(&self, transaction_id: TransactionId) -> bool;
    async fn is_terminated(&self, transaction_id: TransactionId) -> bool;
    async fn is_timedout(&self, transaction_id: TransactionId) -> bool;
    async fn is_errored(&self, transaction_id: TransactionId) -> bool;
}

#[async_trait]
impl TransactionUasInviteExt for sip_server::Transaction {
    async fn is_proceeding(&self, transaction_id: TransactionId) -> bool {
        delay_for(Duration::from_millis(1)).await;
        match self
            .inner
            .state
            .read()
            .await
            .get(&transaction_id)
            .expect("getting transaction from state")
        {
            TrxStateSm::UasInvite(sm) => {
                matches!(
                    sm.lock().await.state,
                    uas_invite::TrxState::Proceeding { .. }
                )
            }
            _ => false,
        }
    }

    async fn is_completed(&self, transaction_id: TransactionId) -> bool {
        delay_for(Duration::from_millis(1)).await;
        match self
            .inner
            .state
            .read()
            .await
            .get(&transaction_id)
            .expect("getting transaction from state")
        {
            TrxStateSm::UasInvite(sm) => {
                matches!(
                    sm.lock().await.state,
                    uas_invite::TrxState::Completed { .. }
                )
            }
            _ => false,
        }
    }

    async fn is_accepted(&self, transaction_id: TransactionId) -> bool {
        delay_for(Duration::from_millis(1)).await;
        match self
            .inner
            .state
            .read()
            .await
            .get(&transaction_id)
            .expect("getting transaction from state")
        {
            TrxStateSm::UasInvite(sm) => {
                matches!(sm.lock().await.state, uas_invite::TrxState::Accepted { .. })
            }
            _ => false,
        }
    }

    async fn is_confirmed(&self, transaction_id: TransactionId) -> bool {
        delay_for(Duration::from_millis(1)).await;
        match self
            .inner
            .state
            .read()
            .await
            .get(&transaction_id)
            .expect("getting transaction from state")
        {
            TrxStateSm::UasInvite(sm) => {
                matches!(
                    sm.lock().await.state,
                    uas_invite::TrxState::Confirmed { .. }
                )
            }
            _ => false,
        }
    }

    async fn is_terminated(&self, transaction_id: TransactionId) -> bool {
        delay_for(Duration::from_millis(1)).await;
        match self
            .inner
            .state
            .read()
            .await
            .get(&transaction_id)
            .expect("getting transaction from state")
        {
            TrxStateSm::UasInvite(sm) => {
                matches!(
                    sm.lock().await.state,
                    uas_invite::TrxState::Terminated(uas_invite::Terminated::Expected { .. })
                )
            }
            _ => false,
        }
    }

    async fn is_timedout(&self, transaction_id: TransactionId) -> bool {
        delay_for(Duration::from_millis(1)).await;
        match self
            .inner
            .state
            .read()
            .await
            .get(&transaction_id)
            .expect("getting transaction from state")
        {
            TrxStateSm::UasInvite(sm) => {
                matches!(
                    sm.lock().await.state,
                    uas_invite::TrxState::Terminated(uas_invite::Terminated::TimedOut { .. })
                )
            }
            _ => false,
        }
    }

    async fn is_errored(&self, transaction_id: TransactionId) -> bool {
        delay_for(Duration::from_millis(1)).await;
        match self
            .inner
            .state
            .read()
            .await
            .get(&transaction_id)
            .expect("getting transaction from state")
        {
            TrxStateSm::UasInvite(sm) => {
                matches!(
                    sm.lock().await.state,
                    uas_invite::TrxState::Terminated(uas_invite::Terminated::Errored { .. })
                )
            }
            _ => false,
        }
    }
}
