use crate::transport::{RequestMsg, ResponseMsg, TransportMsg};
use common::{rsip::Response, tokio::sync::oneshot::Sender};

#[derive(Debug)]
pub enum TransactionLayerMsg {
    NewUacInvite(RequestMsg),                   //from tu
    NewUasInvite(RequestMsg, Option<Response>), //from tu
    Reply(ResponseMsg),                         //from tu
    Incoming(TransportMsg),                     //from transport
    //transaction_id and response channel
    HasTransaction(String, Sender<bool>),       //from transport
}
