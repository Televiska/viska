use crate::common::factories::{common::SocketAddrBuilder, requests::request};
use models::transport::TransportMsg;
use rsip::{common::Transport, SipMessage};
use std::net::SocketAddr;
use crate::common::factories::prelude::*;

pub struct TransportMsgBuilder {
    pub sip_message: SipMessage,
    pub peer: SocketAddr,
    pub transport: Transport,
}

impl Default for TransportMsgBuilder {
    fn default() -> Self {
        Self {
            sip_message: requests::request(None, None).into(),
            peer: SocketAddrBuilder::default().into(),
            transport: Transport::default().into(),
        }
    }
}

impl TransportMsgBuilder {
    //TODO: need to sync peer with sip_message
    pub fn build(self) -> TransportMsg {
        TransportMsg {
            sip_message: self.sip_message,
            peer: self.peer.into(),
            transport: self.transport,
        }
    }
}
