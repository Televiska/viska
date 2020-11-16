use crate::Error;
use models::transport::TransportMsg;
use rsip::SipMessage;

//transport processor

#[derive(Debug, Default)]
pub struct Processor;

impl Processor {
    pub async fn process_incoming_message(&self, msg: TransportMsg) -> Result<TransportMsg, Error> {
        use super::{uac, uas};

        let TransportMsg {
            sip_message,
            peer,
            transport,
        } = msg;

        let sip_message = match sip_message {
            SipMessage::Request(request) => uas::apply_request_defaults(request, peer, transport)?,
            SipMessage::Response(response) => {
                uac::apply_response_defaults(response, peer, transport)?
            }
        };

        Ok(TransportMsg {
            sip_message,
            peer,
            transport,
        })
    }

    pub fn process_outgoing_message(&self, msg: TransportMsg) -> TransportMsg {
        use super::{uac, uas};

        let TransportMsg {
            sip_message,
            peer,
            transport,
        } = msg;

        let sip_message = match sip_message {
            SipMessage::Request(request) => uac::apply_request_defaults(request, peer, transport),
            SipMessage::Response(response) => {
                uas::apply_response_defaults(response, peer, transport)
            }
        };

        TransportMsg {
            sip_message,
            peer,
            transport,
        }
    }
}
