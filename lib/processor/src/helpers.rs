use common::libsip;
use models::SipMessage;

pub fn trace_sip_message(sip_message: SipMessage) -> Result<(), crate::Error> {
    match sip_message.clone() {
        SipMessage::Request(request) => {
            let mut request: store::DirtyRequest = request.into();
            request.raw_message = Some(Into::<libsip::SipMessage>::into(sip_message).to_string());
            store::Request::create(request)?;
        }
        SipMessage::Response(response) => {
            let mut response: store::DirtyResponse = response.into();
            response.raw_message = Some(Into::<libsip::SipMessage>::into(sip_message).to_string());
            store::Response::create(response)?;
        }
    };

    Ok(())
}
