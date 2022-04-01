use crate::Error;
use common::{rsip, tokio::sync::Mutex};
use models::{rsip_ext::*, tu::DialogId, Handlers};

#[derive(Debug)]
pub struct MultiDialog {
    pub id: DialogId,
    dialogs: Mutex<Vec<super::DialogSm>>,
}

impl MultiDialog {
    pub async fn new(handlers: Handlers, msg: rsip::Request) -> Result<Self, Error> {
        Ok(Self {
            id: msg.dialog_id()?,
            dialogs: Mutex::new(vec![super::DialogSm::new(handlers, msg).await?]),
        })
    }

    pub async fn process_incoming_request(&self, msg: rsip::Request) -> Result<(), Error> {
        let dialog_id = msg.dialog_id()?;

        //TODO: decouple the find part, will be needed all over the place
        let mut dialogs = self.dialogs.lock().await;

        let dialog = match dialogs.iter_mut().find(|d| d.id == dialog_id) {
            Some(dialog) => dialog,
            None => dialogs
                .first_mut()
                .expect("No dialog inside MultiDialog Vec ??"),
        };

        dialog.process_incoming_request(msg).await;

        Ok(())
    }

    pub async fn process_incoming_response(&self, msg: rsip::Response) -> Result<(), Error> {
        let dialog_id = msg.dialog_id()?;

        //TODO: decouple the find part, will be needed all over the place
        let mut dialogs = self.dialogs.lock().await;

        let dialog = match dialogs.iter_mut().find(|d| d.id == dialog_id) {
            Some(dialog) => dialog,
            None => dialogs
                .first_mut()
                .expect("No dialog inside MultiDialog Vec ??"),
        };

        dialog.process_incoming_response(msg).await;

        Ok(())
    }

    pub async fn transport_error(&self, reason: String, msg: rsip::SipMessage) {
        let dialog_id = msg.dialog_id().expect("missing dialog_id to report error");

        //TODO: decouple the find part, will be needed all over the place
        let mut dialogs = self.dialogs.lock().await;

        let dialog = match dialogs.iter_mut().find(|d| d.id == dialog_id) {
            Some(dialog) => dialog,
            None => dialogs
                .first_mut()
                .expect("No dialog inside MultiDialog Vec ??"),
        };

        dialog.transport_error(reason, msg).await;
    }
}
