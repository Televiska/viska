mod error;
mod helpers;
mod presets;
mod transactions;

use common::libsip::SipMessage;
pub use error::Error;
use models::{Request, Response};
use std::convert::TryInto;

//should be generic soon
//generic is going to be injected during initialization (no initialization atm)
pub struct Processor;

#[allow(clippy::new_without_default)]
impl Processor {
    pub fn new() -> Self {
        Self
    }

    pub async fn process_message(&self, bytes: common::bytes::BytesMut) -> Result<Vec<u8>, Error> {
        let sip_message: SipMessage = helpers::parse_bytes(bytes.clone())?;
        helpers::trace_sip_message(sip_message.clone(), Some(bytes))?;

        let sip_response: SipMessage = match sip_message {
            SipMessage::Request { .. } => Ok(self.handle_request(sip_message.try_into()?)?.into()),
            SipMessage::Response { .. } => Err(String::from("we don't support responses here")),
        }?;

        helpers::trace_sip_message(sip_response.clone(), None)?;
        Ok(format!("{}", sip_response).into_bytes())
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
        use transactions::DialogExt;

        match dialog {
            Some(dialog) => Ok(dialog.transaction().next(request)?),
            None => {
                let auth_header = request.auth_header();
                match auth_header {
                    Ok(Some(header)) => {
                        if presets::is_authorized(header)? {
                            let dialog: models::Dialog =
                                store::Dialog::create_with_transaction(request.clone())?.into();
                            Ok(dialog.transaction().next(request)?)
                        } else {
                            Ok(presets::create_unauthorized_from(request)?)
                        }
                    }
                    Ok(None) => Ok(presets::create_unauthorized_from(request)?),
                    Err(err) => {
                        common::log::warn!("issue in auth header: {}", err);
                        Ok(presets::create_unauthorized_from(request)?)
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
