pub mod uac;
pub mod uas;

use crate::{error::TransactionError, Error};
use common::{rsip, tokio::sync::Mutex};
use std::fmt::Debug;

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum TrxStateSm {
    Uac(Mutex<uac::TrxStateMachine>),
    Uas(Mutex<uas::TrxStateMachine>),
}

impl TrxStateSm {
    pub async fn is_active(&self) -> bool {
        match self {
            Self::Uac(sm) => sm.lock().await.is_active(),
            Self::Uas(sm) => sm.lock().await.is_active(),
        }
    }

    pub async fn transport_error(&self, reason: String) {
        match self {
            Self::Uac(sm) => sm.lock().await.transport_error(reason).await,
            Self::Uas(sm) => sm.lock().await.transport_error(reason).await,
        };
    }

    pub async fn uac_process_response(&self, msg: rsip::Response) -> Result<(), Error> {
        match self {
            Self::Uac(sm) => {
                let mut sm = sm.lock().await;
                sm.next(Some(msg)).await;
                Ok(())
            }
            Self::Uas(_) => Err(Error::from(TransactionError::UnexpectedState)),
        }
    }

    pub async fn uas_process_request(&self, msg: rsip::Request) -> Result<(), Error> {
        match self {
            Self::Uas(sm) => {
                let mut sm = sm.lock().await;
                sm.next(Some(msg.into())).await;
                Ok(())
            }
            Self::Uac(_) => Err(Error::from(TransactionError::UnexpectedState)),
        }
    }

    pub async fn uas_process_tu_reply(&self, msg: rsip::Response) -> Result<(), Error> {
        match self {
            Self::Uas(sm) => {
                let mut sm = sm.lock().await;
                sm.next(Some(msg.into())).await;
                Ok(())
            }
            Self::Uac(_) => Err(Error::from(TransactionError::UnexpectedState)),
        }
    }
}

impl From<uac::TrxStateMachine> for TrxStateSm {
    fn from(from: uac::TrxStateMachine) -> Self {
        Self::Uac(Mutex::new(from))
    }
}

impl From<uas::TrxStateMachine> for TrxStateSm {
    fn from(from: uas::TrxStateMachine) -> Self {
        Self::Uas(Mutex::new(from))
    }
}
