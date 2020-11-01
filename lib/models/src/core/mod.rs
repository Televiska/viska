use crate::{transaction::TransactionMsg, transport::TransportMsg};
use rsip::{common::Transport, SipMessage};
use std::convert::TryFrom;
use std::net::SocketAddr;

#[derive(Debug)]
pub struct CoreMsg {
    pub sip_message: SipMessage,
    pub peer: SocketAddr,
    pub transport: Transport, //pub ttl: u32
}

impl Into<TransportMsg> for CoreMsg {
    fn into(self) -> TransportMsg {
        TransportMsg {
            sip_message: self.sip_message,
            peer: self.peer,
            transport: self.transport,
        }
    }
}

impl TryFrom<TransportMsg> for CoreMsg {
    type Error = crate::Error;

    fn try_from(transport_msg: TransportMsg) -> Result<Self, Self::Error> {
        Ok(Self {
            sip_message: transport_msg.sip_message,
            peer: transport_msg.peer,
            transport: transport_msg.transport,
        })
    }
}

impl Into<TransactionMsg> for CoreMsg {
    fn into(self) -> TransactionMsg {
        TransactionMsg {
            sip_message: self.sip_message,
            peer: self.peer,
            transport: self.transport,
        }
    }
}

impl TryFrom<TransactionMsg> for CoreMsg {
    type Error = crate::Error;

    fn try_from(transaction_msg: TransactionMsg) -> Result<Self, Self::Error> {
        Ok(Self {
            sip_message: transaction_msg.sip_message,
            peer: transaction_msg.peer,
            transport: transaction_msg.transport,
        })
    }
}
