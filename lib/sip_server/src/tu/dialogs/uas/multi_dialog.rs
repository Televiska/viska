use crate::Error;
use common::{rsip, tokio::sync::Mutex};
use models::Handlers;

#[derive(Debug)]
pub struct MultiDialog {
    dialogs: Mutex<Vec<super::DialogSm>>,
}

impl MultiDialog {
    pub async fn new(handlers: Handlers, msg: rsip::Request) -> Result<Self, Error> {
        Ok(Self {
            dialogs: Mutex::new(vec![super::DialogSm::new(handlers, msg).await?]),
        })
    }

    pub async fn process_response(&self, msg: rsip::Response) -> Result<(), Error> {
        /*
        let dialog_id = msg.dialog_id()?;
        if dialog_id.is_unconfirmed() {
            //TODO: this is not compatible with rfc2543
            return Err(Error::custom(
                "SIP Response in a uas dialog without a tag in TO tag",
            ));
        }

        let mut dialogs = self.dialogs.lock().await;

        let dialog = match dialogs.iter_mut().find(|d| *d == dialog_id) {
            Some(dialog) => dialog,
            None => dialogs
                .first_mut()
                .expect("No dialog inside MultiDialog Vec ??"),
        };

        dialog.process_response(msg.sip_response).await;
        */

        Ok(())
    }

    pub async fn transport_error(&mut self, reason: String, msg: Option<rsip::SipMessage>) {}
}
