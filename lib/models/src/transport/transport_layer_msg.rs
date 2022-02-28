use crate::transport::{RequestMsg, ResponseMsg, TransportMsg, UdpTuple};

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum TransportLayerMsg {
    Outgoing(TransportMsg), //from transaction or tu
    Incoming(UdpTuple),     //from network
}

impl From<TransportMsg> for TransportLayerMsg {
    fn from(from: TransportMsg) -> Self {
        Self::Outgoing(from)
    }
}

impl From<RequestMsg> for TransportLayerMsg {
    fn from(from: RequestMsg) -> Self {
        Self::Outgoing(from.into())
    }
}

impl From<ResponseMsg> for TransportLayerMsg {
    fn from(from: ResponseMsg) -> Self {
        Self::Outgoing(from.into())
    }
}

impl From<UdpTuple> for TransportLayerMsg {
    fn from(from: UdpTuple) -> Self {
        Self::Incoming(from)
    }
}
