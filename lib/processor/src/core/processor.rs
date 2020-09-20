pub use crate::Error;
use models::{Request, Response, SipMessage};

pub struct Processor;

#[allow(clippy::new_without_default)]
impl Processor {
    pub fn new() -> Self {
        Self
    }

    pub async fn process_message(&self, sip_message: SipMessage) -> Result<SipMessage, Error> {
        crate::helpers::trace_sip_message(sip_message.clone())?;

        let sip_message: SipMessage = match sip_message {
            SipMessage::Request(request) => self.handle_request(request),
            SipMessage::Response(_) => Err(Error::from("we don't support responses yet")),
        }?
        .into();

        crate::helpers::trace_sip_message(sip_message.clone())?;
        Ok(sip_message)
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
                let auth_header = request.auth_header();
                match auth_header {
                    Ok(Some(header)) => {
                        if crate::presets::is_authorized(header)? {
                            let dialog: models::Dialog =
                                store::Dialog::create_with_transaction(request.clone())?.into();
                            Ok(dialog.transaction().next(request)?)
                        } else {
                            Ok(crate::presets::create_unauthorized_from(request)?)
                        }
                    }
                    Ok(None) => Ok(crate::presets::create_unauthorized_from(request)?),
                    Err(err) => {
                        common::log::warn!("issue in auth header: {}", err);
                        Ok(crate::presets::create_unauthorized_from(request)?)
                    }
                }
            }
        }
    }

    fn dialog_from(&self, request: Request) -> Option<models::Dialog> {
        store::Dialog::find_with_transaction(request.dialog_id()?)
            .ok()
            .map(|s| s.into())
    }
}
