use crate::transport::TransportMsg;
use rsip::{common::Transport, SipMessage};
use std::convert::TryFrom;
use std::net::SocketAddr;

#[derive(Debug)]
pub struct TransactionMsg {
    pub sip_message: SipMessage,
    pub peer: SocketAddr,
    pub transport: Transport, //pub ttl: u32
}

impl Into<TransportMsg> for TransactionMsg {
    fn into(self) -> TransportMsg {
        TransportMsg {
            sip_message: self.sip_message,
            peer: self.peer,
            transport: self.transport,
        }
    }
}

impl TryFrom<TransportMsg> for TransactionMsg {
    type Error = crate::Error;

    fn try_from(transport_msg: TransportMsg) -> Result<Self, Self::Error> {
        Ok(Self {
            sip_message: transport_msg.sip_message,
            peer: transport_msg.peer,
            transport: transport_msg.transport,
        })
    }
}
