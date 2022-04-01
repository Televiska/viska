use common::{rsip, tokio::sync::oneshot::Sender};

#[derive(Debug)]
pub enum TransactionLayerMsg {
    NewUacInvite(rsip::Request),                         //from tu
    NewUasInvite(rsip::Request, Option<rsip::Response>), //from tu
    NewUac(rsip::Request),                               //from tu
    NewUas(rsip::Request, Option<rsip::Response>),       //from tu
    Reply(rsip::Response),                               //from tu
    Incoming(rsip::SipMessage),                          //from transport
    TransportError(rsip::SipMessage, TransportError),
    HasTransaction(TransactionId, Sender<bool>), //from transport
}

//TODO: add proper error type here
pub type TransportError = String;
//TODO: add proper (rsip) type here
pub type TransactionId = rsip::param::Branch;
