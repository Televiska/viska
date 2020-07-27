mod helpers;
mod transactions;

use common::libsip::SipMessage;
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

    pub async fn process_message(&self, bytes: common::bytes::BytesMut) -> Result<Vec<u8>, String> {
        let sip_message: SipMessage = helpers::parse_bytes(bytes.clone())?;
        helpers::trace_sip_message(sip_message.clone(), Some(bytes));

        let sip_response: SipMessage = match sip_message {
            SipMessage::Request { .. } => Ok(self.handle_request(sip_message.try_into()?)?.into()),
            SipMessage::Response { .. } => Err(String::from("we don't support responses here")),
        }?;

        helpers::trace_sip_message(sip_response.clone(), None);
        Ok(format!("{}", sip_response).into_bytes())
    }

    fn handle_request(&self, request: Request) -> Result<Response, String> {
        let response = self.handle_next_step_for(self.dialog_from(request.clone())?, request)?;

        Ok(response)
    }

    fn handle_next_step_for(
        &self,
        dialog: models::Dialog,
        request: Request,
    ) -> Result<Response, String> {
        use transactions::DialogExt;

        Ok(dialog.transaction().next(request)?)
    }

    fn dialog_from(&self, request: Request) -> Result<models::Dialog, String> {
        Ok(store::Dialog::find_or_create_dialog(request)
            .map_err(|e| e.to_string())?
            .into())
    }
}
