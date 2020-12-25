use crate::{server::UdpTuple, transport::TransportMsg, Error, SipMessageExt};
use rsip::common::Transport;
use std::convert::{TryFrom, TryInto};
use std::net::SocketAddr;

//TODO: we probably need better naming here
#[derive(Debug, Clone)]
pub struct RequestMsg {
    pub sip_request: rsip::Request,
    pub peer: SocketAddr,
    pub transport: Transport, //pub ttl: u32
}

impl RequestMsg {
    pub fn new(sip_request: rsip::Request, peer: SocketAddr, transport: Transport) -> Self {
        RequestMsg {
            sip_request,
            peer,
            transport,
        }
    }

    pub fn transaction_id(&self) -> Result<String, Error> {
        SipMessageExt::transaction_id(&self.sip_request)
    }
}

impl Into<UdpTuple> for RequestMsg {
    fn into(self) -> UdpTuple {
        UdpTuple {
            bytes: self.sip_request.into(),
            peer: self.peer,
        }
    }
}

impl TryFrom<UdpTuple> for RequestMsg {
    type Error = crate::Error;

    fn try_from(udp_tuple: UdpTuple) -> Result<Self, Self::Error> {
        Ok(Self {
            sip_request: udp_tuple.bytes.try_into()?,
            peer: udp_tuple.peer,
            transport: Transport::Udp,
        })
    }
}

impl Into<TransportMsg> for RequestMsg {
    fn into(self) -> TransportMsg {
        TransportMsg {
            sip_message: self.sip_request.into(),
            peer: self.peer,
            transport: self.transport,
        }
    }
}

impl TryFrom<TransportMsg> for RequestMsg {
    type Error = crate::Error;

    fn try_from(transport_msg: TransportMsg) -> Result<RequestMsg, Self::Error> {
        Ok(RequestMsg {
            sip_request: transport_msg.sip_message.try_into()?,
            peer: transport_msg.peer,
            transport: transport_msg.transport,
        })
    }
}
