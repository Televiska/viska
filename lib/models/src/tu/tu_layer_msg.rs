use crate::transport::TransportMsg;

//TODO: probably makes sense to split incoming from transport
//and incoming from transaction
#[derive(Debug, Clone)]
pub enum TuLayerMsg {
    Incoming(TransportMsg),
    Outgoing(TransportMsg),
    TransportError(TransportMsg, TransportError),
}

//TODO: add proper error type here
pub type TransportError = String;
