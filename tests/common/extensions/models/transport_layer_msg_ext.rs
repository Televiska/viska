use common::rsip;
use models::transport::TransportLayerMsg;
use std::convert::TryInto;

pub trait TransportLayerMsgExt {
    fn outgoing_msg(&self) -> rsip::SipMessage;
    fn outgoing_request(&self) -> rsip::Request;
    fn outgoing_response(&self) -> rsip::Response;
}

impl TransportLayerMsgExt for TransportLayerMsg {
    fn outgoing_msg(&self) -> rsip::SipMessage {
        match self {
            TransportLayerMsg::Outgoing(msg) => msg.clone(),
            _ => panic!("not an Outgoing variant"),
        }
    }

    fn outgoing_request(&self) -> rsip::Request {
        match self {
            TransportLayerMsg::Outgoing(request) => {
                request.clone().try_into().expect("into rsip::Request")
            }
            _ => panic!("not an Outgoing variant"),
        }
    }

    fn outgoing_response(&self) -> rsip::Response {
        match self {
            TransportLayerMsg::Outgoing(response) => {
                response.clone().try_into().expect("into rsip::Response")
            }
            _ => panic!("not an Outgoing variant"),
        }
    }
}
