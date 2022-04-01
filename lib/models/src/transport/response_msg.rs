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
pub struct ResponseMsg {
    pub sip_response: rsip::Response,
    pub peer: SocketAddr,
    pub transport: Transport, //pub ttl: u32
}

impl ResponseMsg {
    pub fn new(sip_response: rsip::Response, peer: SocketAddr, transport: Transport) -> Self {
        ResponseMsg {
            sip_response,
            peer,
            transport,
        }
    }

    //TODO: should be a proper type here instead of simply String
    pub fn transaction_id(&self) -> Result<Option<String>, Error> {
        Ok(self.sip_response.transaction_id()?.map(Into::into))
    }

    pub fn dialog_id(&self) -> Result<DialogId, Error> {
        self.sip_response.dialog_id()
    }
}

impl From<(rsip::Response, SocketAddr, Transport)> for ResponseMsg {
    fn from(triple: (rsip::Response, SocketAddr, Transport)) -> Self {
        Self {
            sip_response: triple.0,
            peer: triple.1,
            transport: triple.2,
        }
    }
}

impl TryFrom<UdpTuple> for ResponseMsg {
    type Error = crate::Error;

    fn try_from(udp_tuple: UdpTuple) -> Result<Self, Self::Error> {
        Ok(Self {
            sip_response: udp_tuple.bytes.try_into()?,
            peer: udp_tuple.peer,
            transport: Transport::Udp,
        })
    }
}

impl TryFrom<TransportMsg> for ResponseMsg {
    type Error = crate::Error;

    fn try_from(transport_msg: TransportMsg) -> Result<ResponseMsg, Self::Error> {
        Ok(ResponseMsg {
            sip_response: transport_msg.sip_message.try_into()?,
            peer: transport_msg.peer,
            transport: transport_msg.transport,
        })
    }
}
