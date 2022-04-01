use common::rsip;
use models::transport::{RequestMsg, ResponseMsg, TransportLayerMsg, TransportMsg};
use std::convert::TryInto;

pub trait TransportLayerMsgExt {
    fn outgoing_msg(&self) -> TransportMsg;
    fn outgoing_request_msg(&self) -> RequestMsg;
    fn outgoing_response_msg(&self) -> ResponseMsg;
    fn outgoing_sip_request(&self) -> rsip::Request {
        self.outgoing_request_msg().sip_request
    }
    fn outgoing_sip_response(&self) -> rsip::Response {
        self.outgoing_response_msg().sip_response
    }
}

impl TransportLayerMsgExt for TransportLayerMsg {
    fn outgoing_msg(&self) -> TransportMsg {
        match self {
            TransportLayerMsg::Outgoing(transport_msg) => transport_msg.clone(),
            _ => panic!("not an Outgoing variant"),
        }
    }

    fn outgoing_request_msg(&self) -> RequestMsg {
        match self {
            TransportLayerMsg::Outgoing(transport_msg) => {
                transport_msg.clone().try_into().expect("into RequestMsg")
            }
            _ => panic!("not an Outgoing variant"),
        }
    }

    fn outgoing_response_msg(&self) -> ResponseMsg {
        match self {
            TransportLayerMsg::Outgoing(transport_msg) => {
                transport_msg.clone().try_into().expect("into ResponseMsg")
            }
            _ => panic!("not an Outgoing variant"),
        }
    }
}
