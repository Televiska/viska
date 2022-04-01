use crate::{
    rsip_ext::DialogExt,
    transport::{TransportMsg, UdpTuple},
    tu::DialogId,
    Error,
};
use common::rsip::{self, prelude::*, Transport};
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

    pub fn transaction_id(&self) -> Result<Option<String>, Error> {
        Ok(self.sip_request.transaction_id()?.map(Into::into))
    }

    pub fn dialog_id(&self) -> Result<DialogId, Error> {
        self.sip_request.dialog_id()
    }
}

impl From<(rsip::Request, SocketAddr, Transport)> for RequestMsg {
    fn from(triple: (rsip::Request, SocketAddr, Transport)) -> Self {
        Self {
            sip_request: triple.0,
            peer: triple.1,
            transport: triple.2,
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
