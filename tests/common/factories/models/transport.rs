use crate::common::factories::{
    common::{SocketAddr, Transport},
    requests::request,
};
use models::{transport::TransportMsg, SipMessage, TransportType};

pub struct TransportMsgBuilder {
    pub sip_message: SipMessage,
    pub peer: SocketAddr,
    pub transport: TransportType,
}

impl Default for TransportMsgBuilder {
    fn default() -> Self {
        Self {
            sip_message: request(None, None).into(),
            peer: SocketAddr::default(),
            transport: Transport::default().into(),
        }
    }
}

impl TransportMsgBuilder {
    //TODO: need to sync peer with sip_message
    pub fn build(self) -> TransportMsg {
        TransportMsg {
            sip_message: self.sip_message,
            peer: self.peer.into(),
            transport: self.transport,
        }
    }
}
