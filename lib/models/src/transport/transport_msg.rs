use crate::transport::{RequestMsg, ResponseMsg};
use crate::{server::UdpTuple, Error};
use common::rsip::{common::Transport, prelude::*};
use std::convert::{TryFrom, TryInto};
use std::net::SocketAddr;

//TODO: we probably need better naming here
#[derive(Debug, Clone)]
pub struct TransportMsg {
    pub sip_message: rsip::SipMessage,
    pub peer: SocketAddr,
    pub transport: Transport, //pub ttl: u32
}

impl TransportMsg {
    pub fn transaction_id(&self) -> Result<String, Error> {
        Ok(self.sip_message.transaction_id()?)
    }
}

impl From<(rsip::SipMessage, SocketAddr, Transport)> for TransportMsg {
    fn from(triple: (rsip::SipMessage, SocketAddr, Transport)) -> Self {
        Self {
            sip_message: triple.0,
            peer: triple.1,
            transport: triple.2,
        }
    }
}

impl TryFrom<UdpTuple> for TransportMsg {
    type Error = crate::Error;

    fn try_from(udp_tuple: UdpTuple) -> Result<Self, Self::Error> {
        Ok(Self {
            sip_message: udp_tuple.bytes.try_into()?,
            peer: udp_tuple.peer,
            transport: Transport::Udp,
        })
    }
}

impl From<RequestMsg> for TransportMsg {
    fn from(from: RequestMsg) -> Self {
        TransportMsg {
            sip_message: from.sip_request.into(),
            peer: from.peer,
            transport: from.transport,
        }
    }
}

impl From<ResponseMsg> for TransportMsg {
    fn from(from: ResponseMsg) -> Self {
        TransportMsg {
            sip_message: from.sip_response.into(),
            peer: from.peer,
            transport: from.transport,
        }
    }
}
