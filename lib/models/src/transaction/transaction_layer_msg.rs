use crate::transport::{RequestMsg, ResponseMsg, TransportMsg};
use common::{rsip::Response, tokio::sync::oneshot::Sender};

#[derive(Debug)]
pub enum TransactionLayerMsg {
    NewUacInvite(RequestMsg),                   //from tu
    NewUasInvite(RequestMsg, Option<Response>), //from tu
    Reply(ResponseMsg),                         //from tu
    Incoming(TransportMsg),                     //from transport
    TransportError(TransportMsg, TransportError),
    HasTransaction(TransactionId, Sender<bool>), //from transport
}

//TODO: add proper error type here
pub type TransportError = String;
//TODO: add proper (rsip) type here
pub type TransactionId = String;
