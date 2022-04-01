use common::rsip;

//TODO: probably makes sense to split incoming from transport
//and incoming from transaction
#[derive(Debug, Clone)]
pub enum TuLayerMsg {
    Incoming(rsip::SipMessage),
    Outgoing(rsip::SipMessage),
    TransportError(rsip::SipMessage, TransportError),
}

//TODO: add proper error type here
pub type TransportError = String;
