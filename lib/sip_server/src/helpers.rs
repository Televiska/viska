use common::rsip::{self};

pub fn trace_sip_message(sip_message: rsip::SipMessage) -> Result<(), crate::Error> {
    match sip_message.clone() {
        rsip::SipMessage::Request(request) => {
            let mut request: store::DirtyRequest = request.into();
            request.raw_message = Some(Into::<String>::into(sip_message));
            store::Request::create(request)?;
        }
        rsip::SipMessage::Response(response) => {
            let mut response: store::DirtyResponse = response.into();
            response.raw_message = Some(Into::<String>::into(sip_message));
            store::Response::create(response)?;
        }
    };

    Ok(())
}
