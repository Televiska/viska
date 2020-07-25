mod helpers;

use common::libsip::SipMessage;
use models::{Request, Response};
use std::convert::TryInto;

pub async fn process_message(bytes: common::bytes::BytesMut) -> Result<Vec<u8>, String> {
    let sip_message: SipMessage = helpers::parse_bytes(bytes.clone())?;
    helpers::trace_sip_message(sip_message.clone(), Some(bytes)).await;

    let sip_response: SipMessage = match sip_message {
        SipMessage::Request { .. } => Ok(handle_request(sip_message.try_into()?).await?.into()),
        SipMessage::Response { .. } => Err(String::from("we don't support responses here")),
    }?;

    helpers::trace_sip_message(sip_response.clone(), None).await;
    Ok(format!("{}", sip_response).into_bytes())
}

async fn handle_request(request: Request) -> Result<Response, String> {
    let response = handle_next_step_for(state_from(request).await?)?;

    Ok(response)
}

fn handle_next_step_for(state: models::ServerState) -> Result<Response, String> {
    use models::DialogExt;

    Ok(state.dialog.transaction().next(state.request)?)
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
