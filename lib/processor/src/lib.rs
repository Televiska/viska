use common::libsip::SipMessage;
use models::{Request, Response};
use std::convert::TryInto;

pub async fn get_response(msg: SipMessage) -> Result<SipMessage, String> {
    match msg {
        SipMessage::Request { .. } => Ok(handle_request(msg.try_into()?).await?.into()),
        SipMessage::Response { .. } => Err("we don't support responses yet".into()),
    }
}

async fn handle_request(request: Request) -> Result<Response, String> {
    Ok(handle_next_step_for(state_from(request).await?)?)
}

fn handle_next_step_for(state: models::ServerState) -> Result<Response, String> {
    match state.dialog.transaction() {
        models::TransactionType::Transaction(tr) => Ok(tr.next(state.request)?),
        models::TransactionType::NotFound(tr) => Ok(tr.next(state.request)?),
    }
}

async fn state_from(request: Request) -> Result<models::ServerState, String> {
    Ok(models::ServerState {
        dialog: find_or_create_dialog(request.clone()).await?,
        request,
    })
}

async fn find_or_create_dialog(request: Request) -> Result<models::Dialog, String> {
    match request.dialog_id() {
        Some(dialog_id) => Ok(store::Dialog::find_with_transaction(dialog_id)
            .await
            .map_err(|e| e.to_string())?
            .into()),
        None => Ok(
            store::Dialog::create_with_transaction(request.clone().try_into()?)
                .await
                .map_err(|e| e.to_string())?
                .into(),
        ),
    }
}
