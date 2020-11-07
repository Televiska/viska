use crate::common::factories::prelude::*;
use crate::common::factories::prelude::*;
use models::transport::TransportMsg;
use rsip::{common::Transport, SipMessage};
use std::net::SocketAddr;

impl Randomized for TransportMsg {
    fn default() -> Self {
        Self {
            sip_message: factories::requests::request(None, None).into(),
            peer: SocketAddrBuilder::default().into(),
            transport: Transport::default(),
        }
    }
}
