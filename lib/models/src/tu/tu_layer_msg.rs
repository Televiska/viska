use crate::transport::{RequestMsg, ResponseMsg, TransportMsg};

//TODO: probably makes sense to split incoming from transport
//and incoming from transaction
#[derive(Debug, Clone)]
pub enum TuLayerMsg {
    Incoming(TransportMsg),
}

impl From<TransportMsg> for TuLayerMsg {
    fn from(from: TransportMsg) -> Self {
        Self::Incoming(from)
    }
}

impl From<RequestMsg> for TuLayerMsg {
    fn from(from: RequestMsg) -> Self {
        Self::Incoming(from.into())
    }
}

impl From<ResponseMsg> for TuLayerMsg {
    fn from(from: ResponseMsg) -> Self {
        Self::Incoming(from.into())
    }
}
