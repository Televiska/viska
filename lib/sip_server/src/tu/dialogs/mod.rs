pub mod uac;
//pub mod uas;
pub mod dialog_sm;

pub use crate::error::{DialogError, Error};
use common::{rsip, tokio::sync::RwLock};
use dialog_sm::DialogSm;
use models::{rsip_ext::*, tu::DialogId, Handlers};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Dialogs {
    handlers: Handlers,
    //TODO: convert to message passing
    data: RwLock<HashMap<DialogId, DialogSm>>,
}

impl Dialogs {
    pub fn new(handlers: Handlers) -> Self {
        Self {
            handlers,
            data: Default::default(),
        }
    }

    //TODO: add proper dialog id type
    pub async fn exists(&self, dialog_id: DialogId) -> bool {
        self.data.read().await.get(&dialog_id).is_some()
    }

    pub async fn new_uac_session(&self, msg: rsip::Request) -> Result<(), Error> {
        let dialog_data = uac::MultiDialog::new(self.handlers.clone(), msg).await?;
        let mut data = self.data.write().await;
        data.insert(dialog_data.id.clone(), dialog_data.into());

        Ok(())
    }

    pub async fn process_incoming_response(&self, msg: rsip::Response) -> Result<(), Error> {
        let dialog_id = msg.dialog_id()?;

        if let Some(sm) = self.data.read().await.get(&dialog_id) {
            sm.process_incoming_response(msg).await
        } else {
            Err(Error::from(DialogError::NotFound))
        }
    }

    pub async fn process_incoming_request(&self, msg: rsip::Request) -> Result<(), Error> {
        let dialog_id = msg.dialog_id()?;

        if let Some(sm) = self.data.read().await.get(&dialog_id) {
            sm.process_incoming_request(msg).await
        } else {
            Err(Error::from(DialogError::NotFound))
        }
    }

    //TODO: maybe take a dialog_id here ?
    pub async fn transport_error(
        &self,
        msg: rsip::SipMessage,
        reason: String,
    ) -> Result<(), Error> {
        if let Some(sm) = self.data.read().await.get(&msg.dialog_id()?) {
            sm.transport_error(reason, msg).await;
            Ok(())
        } else {
            Err(Error::from(DialogError::NotFound))
        }
    }
}
