use crate::transport::{RequestMsg, ResponseMsg, TransportMsg};
use common::bytes::Bytes;
use std::net::SocketAddr;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UdpTuple {
    pub bytes: Bytes,
    pub peer: SocketAddr,
}

impl From<(Bytes, SocketAddr)> for UdpTuple {
    fn from(tuple: (Bytes, SocketAddr)) -> Self {
        Self {
            bytes: tuple.0,
            peer: tuple.1,
        }
    }
}

impl From<UdpTuple> for (Bytes, SocketAddr) {
    fn from(udp_tuple: UdpTuple) -> Self {
        (udp_tuple.bytes, udp_tuple.peer)
    }
}

impl From<RequestMsg> for UdpTuple {
    fn from(from: RequestMsg) -> Self {
        UdpTuple {
            bytes: from.sip_request.into(),
            peer: from.peer,
        }
    }
}

impl From<ResponseMsg> for UdpTuple {
    fn from(from: ResponseMsg) -> Self {
        Self {
            bytes: from.sip_response.into(),
            peer: from.peer,
        }
    }
}

impl From<TransportMsg> for UdpTuple {
    fn from(from: TransportMsg) -> Self {
        UdpTuple {
            bytes: from.sip_message.into(),
            peer: from.peer,
        }
    }
}
