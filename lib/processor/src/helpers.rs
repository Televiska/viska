use common::nom::error::VerboseError;
use common::{
    libsip::{self, SipMessage},
    log,
};
use std::convert::TryInto;

pub fn parse_bytes(bytes: common::bytes::BytesMut) -> Result<SipMessage, String> {
    let (_, request) =
        libsip::parse_message::<VerboseError<&[u8]>>(&bytes.to_vec()).map_err(|e| e.to_string())?;

    Ok(request)
}

pub fn trace_sip_message(sip_message: SipMessage, bytes: Option<common::bytes::BytesMut>) {
    let raw_message = match bytes {
        Some(bytes) => String::from_utf8_lossy(&bytes.to_vec()).to_string(),
        None => format!("{}", sip_message),
    };

    match sip_message {
        SipMessage::Request { .. } => {
            let mut request: store::DirtyRequest =
                TryInto::<models::Request>::try_into(sip_message)
                    .expect("should never happen")
                    .into();
            request.raw_message = Some(raw_message);
            store::Request::create(request)
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
                .map_err(|err| log::error!("{}", err))
                .unwrap();
        }
    }
}
