use common::libsip::SipMessage;
use models::{Request, Response};
use std::convert::TryInto;

pub fn get_response(msg: SipMessage) -> Result<SipMessage, String> {
    match msg {
        SipMessage::Request { .. } => Ok(handle_request(msg.try_into()?)?.into()),
        SipMessage::Response { .. } => Err("we don't support responses yet".into()),
    }
}

fn handle_request(request: Request) -> Result<Response, String> {
    Ok(handle_next_step_for(state_from(request)?)?)
}

fn handle_next_step_for(state: models::ServerState) -> Result<Response, String> {
    match state.dialog.transaction() {
        models::TransactionType::Transaction(tr) => Ok(tr.next(state.request)?),
        models::TransactionType::NotFound(tr) => Ok(tr.next(state.request)?),
    }
}

fn state_from(request: Request) -> Result<models::ServerState, String> {
    Ok(models::ServerState {
        dialog: find_or_create_dialog(request.clone())?,
        request,
    })
}

fn find_or_create_dialog(request: Request) -> Result<models::Dialog, String> {
    match request.dialog_id() {
        Some(dialog_id) => Ok(store::Dialogs::find_with_transaction(dialog_id)
            .ok_or("We couldn't find dialog!")?
            .into()),
        None => Ok(store::Dialogs::create(request.clone().try_into()?).into()),
    }
}
