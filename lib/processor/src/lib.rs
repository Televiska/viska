use common::nom::error::VerboseError;
use common::{
    libsip::{self, SipMessage},
    log,
};
use models::{Request, Response};
use std::convert::TryInto;

pub async fn process_message(bytes: common::bytes::BytesMut) -> Result<Vec<u8>, String> {
    let sip_message: SipMessage = parse_bytes(bytes.clone())?;
    trace_sip_message(sip_message.clone(), Some(bytes)).await;

    let sip_response: SipMessage = match sip_message {
        SipMessage::Request { .. } => Ok(handle_request(sip_message.try_into()?).await?.into()),
        SipMessage::Response { .. } => Err(String::from("we don't support responses here")),
    }?;

    trace_sip_message(sip_response.clone(), None).await;
    Ok(format!("{}", sip_response).into_bytes())
}

fn parse_bytes(bytes: common::bytes::BytesMut) -> Result<SipMessage, String> {
    let (_, request) =
        libsip::parse_message::<VerboseError<&[u8]>>(&bytes.to_vec()).map_err(|e| e.to_string())?;

    Ok(request)
}

async fn trace_sip_message(sip_message: SipMessage, bytes: Option<common::bytes::BytesMut>) {
    let raw_message = match bytes {
        Some(bytes) => String::from_utf8_lossy(&bytes.to_vec()).to_string(),
        None => format!("{}", sip_message),
    };

    match sip_message {
        SipMessage::Request { .. } => {
            let mut request: store::DirtyRequest =
                TryInto::<models::Request>::try_into(sip_message.clone())
                    .expect("should never happen")
                    .into();
            request.raw_message = Some(raw_message);
            store::Request::create(request)
                .await
                .map_err(|err| log::error!("{}", err))
                .unwrap();
        }
        SipMessage::Response { .. } => {
            let mut response: store::DirtyResponse =
                TryInto::<models::Response>::try_into(sip_message)
                    .expect("should never happen")
                    .into();
            response.raw_message = Some(raw_message);
            store::Response::create(response)
                .await
                .map_err(|err| log::error!("{}", err))
                .unwrap();
        }
    }
}

async fn handle_request(request: Request) -> Result<Response, String> {
    let response = handle_next_step_for(state_from(request).await?)?;

    Ok(response)
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
