pub use crate::{Error, SipManager};
use models::transport::TransportMsg;
use rsip::{Request, Response, SipMessage};
use std::sync::{Arc, Weak};

pub struct Processor {
    sip_manager: Weak<SipManager>,
}

#[allow(clippy::new_without_default)]
impl Processor {
    pub fn new(sip_manager: Weak<SipManager>) -> Self {
        Self { sip_manager }
    }

    //TODO: Fix me
    pub async fn process_message(&self, msg: TransportMsg) -> Result<(), Error> {
        let sip_message = msg.sip_message;
        //crate::helpers::trace_sip_message(sip_message.clone())?;

        let sip_message: SipMessage = match sip_message {
            SipMessage::Request(request) => self.handle_request(request),
            SipMessage::Response(_) => Err(Error::from("we don't support responses yet")),
        }?
        .into();

        //crate::helpers::trace_sip_message(sip_message.clone())?;
        self.sip_manager()
            .core
            .send(TransportMsg {
                sip_message,
                peer: msg.peer,
                transport: msg.transport,
            })
            .await;

        Ok(())
    }

    fn handle_request(&self, request: Request) -> Result<Response, Error> {
        let response = self.handle_next_step_for(self.dialog_from(request.clone()), request)?;

        Ok(response)
    }

    fn handle_next_step_for(
        &self,
        dialog: Option<models::Dialog>,
        request: Request,
    ) -> Result<Response, Error> {
        use crate::transactions::DialogExt;

        match dialog {
            Some(dialog) => Ok(dialog.transaction().next(request)?),
            None => {
                let auth_header = request.authorization_header();
                match auth_header {
                    Some(header) => {
                        if crate::presets::is_authorized(header.clone())? {
                            let dialog: models::Dialog =
                                store::Dialog::create_with_transaction(request.clone())?.into();
                            Ok(dialog.transaction().next(request)?)
                        } else {
                            Ok(crate::presets::create_unauthorized_from(request)?)
                        }
                    }
                    None => {
                        common::log::warn!("auth header is missing");
                        Ok(crate::presets::create_unauthorized_from(request)?)
                    }
                }
            }
        }
    }

    fn dialog_from(&self, request: Request) -> Option<models::Dialog> {
        use rsip::message::HeadersExt;

        store::Dialog::find_with_transaction(request.dialog_id()?)
            .ok()
            .map(|s| s.into())
    }

    fn sip_manager(&self) -> Arc<SipManager> {
        self.sip_manager.upgrade().expect("sip manager is missing!")
    }
}
