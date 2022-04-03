pub mod uac;
pub mod uac_invite;
pub mod uas_invite;

use crate::{error::TransactionError, Error};
use common::{rsip, tokio::sync::Mutex};
use models::{transaction::TransactionId, Handlers};
use std::fmt::Debug;

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum TrxStateSm {
    UacInvite(Mutex<uac_invite::TrxStateMachine>),
    Uac(Mutex<uac::TrxStateMachine>),
    UasInvite(Mutex<uas_invite::TrxStateMachine>),
}

impl TrxStateSm {
    pub fn new_uac_invite(handlers: Handlers, request: rsip::Request) -> Result<Self, Error> {
        Ok(Self::UacInvite(Mutex::new(
            uac_invite::TrxStateMachine::new(handlers, request)?,
        )))
    }

    pub fn new_uac(handlers: Handlers, request: rsip::Request) -> Result<Self, Error> {
        Ok(Self::Uac(Mutex::new(uac::TrxStateMachine::new(
            handlers, request,
        )?)))
    }

    pub fn new_uas_invite(
        handlers: Handlers,
        request: rsip::Request,
        response: Option<rsip::Response>,
    ) -> Result<Self, Error> {
        Ok(Self::UasInvite(Mutex::new(
            uas_invite::TrxStateMachine::new(handlers, request, response)?,
        )))
    }

    pub async fn id(&self) -> TransactionId {
        match self {
            Self::UacInvite(sm) => sm.lock().await.id.clone(),
            Self::Uac(sm) => sm.lock().await.id.clone(),
            Self::UasInvite(sm) => sm.lock().await.id.clone(),
        }
    }

    pub async fn is_active(&self) -> bool {
        match self {
            Self::UacInvite(sm) => sm.lock().await.is_active(),
            Self::Uac(sm) => sm.lock().await.is_active(),
            Self::UasInvite(sm) => sm.lock().await.is_active(),
        }
    }

    pub async fn transport_error(&self, reason: String) {
        match self {
            Self::UacInvite(sm) => sm.lock().await.transport_error(reason).await,
            Self::Uac(sm) => sm.lock().await.transport_error(reason).await,
            Self::UasInvite(sm) => sm.lock().await.transport_error(reason).await,
        };
    }

    //only UAC process a response in the transaction layer
    pub async fn process_response(&self, msg: rsip::Response) -> Result<(), Error> {
        match self {
            Self::UacInvite(sm) => {
                let mut sm = sm.lock().await;
                sm.next(Some(msg)).await;
                Ok(())
            }
            Self::Uac(sm) => {
                let mut sm = sm.lock().await;
                sm.next(Some(msg)).await;
                Ok(())
            }
            _ => Err(Error::from(TransactionError::UnexpectedState)),
        }
    }

    //only UAS process a request in the transaction layer
    pub async fn process_request(&self, msg: rsip::Request) -> Result<(), Error> {
        match self {
            Self::UasInvite(sm) => {
                let mut sm = sm.lock().await;
                sm.next(Some(msg.into())).await;
                Ok(())
            }
            _ => Err(Error::from(TransactionError::UnexpectedState)),
        }
    }

    //only UAS process a reply from tU in the transaction layer
    pub async fn process_tu_reply(&self, msg: rsip::Response) -> Result<(), Error> {
        match self {
            Self::UasInvite(sm) => {
                let mut sm = sm.lock().await;
                sm.next(Some(msg.into())).await;
                Ok(())
            }
            _ => Err(Error::from(TransactionError::UnexpectedState)),
        }
    }
}
