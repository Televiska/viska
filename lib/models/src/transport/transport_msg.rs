use crate::{server::UdpTuple, Error, SipMessageExt};
use rsip::common::Transport;
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
        SipMessageExt::transaction_id(&self.sip_message)
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

impl Into<UdpTuple> for TransportMsg {
    fn into(self) -> UdpTuple {
        UdpTuple {
            bytes: self.sip_message.into(),
            peer: self.peer,
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
