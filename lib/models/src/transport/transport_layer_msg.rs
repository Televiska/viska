use crate::transport::UdpTuple;
use common::rsip;

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum TransportLayerMsg {
    Outgoing(rsip::SipMessage), //from transaction or tu
    Incoming(UdpTuple),         //from network
}

impl From<rsip::SipMessage> for TransportLayerMsg {
    fn from(from: rsip::SipMessage) -> Self {
        Self::Outgoing(from)
    }
}

impl From<rsip::Request> for TransportLayerMsg {
    fn from(from: rsip::Request) -> Self {
        Self::Outgoing(from.into())
    }
}

impl From<rsip::Response> for TransportLayerMsg {
    fn from(from: rsip::Response) -> Self {
        Self::Outgoing(from.into())
    }
}

impl From<UdpTuple> for TransportLayerMsg {
    fn from(from: UdpTuple) -> Self {
        Self::Incoming(from)
    }
}
