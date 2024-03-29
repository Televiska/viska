use crate::common::factories::prelude::*;
use common::rsip::Transport;
use models::transport::{RequestMsg, ResponseMsg, TransportMsg};

impl Randomized for TransportMsg {
    fn default() -> Self {
        Self {
            sip_message: factories::requests::request(None, None).into(),
            peer: SocketAddrBuilder::default().into(),
            transport: Transport::default(),
        }
    }
}

impl Randomized for RequestMsg {
    fn default() -> Self {
        Self {
            sip_request: factories::requests::request(None, None),
            peer: SocketAddrBuilder::default().into(),
            transport: Transport::default(),
        }
    }
}

impl Randomized for ResponseMsg {
    fn default() -> Self {
        Self {
            sip_response: factories::responses::response(None, None),
            peer: SocketAddrBuilder::default().into(),
            transport: Transport::default(),
        }
    }
}
