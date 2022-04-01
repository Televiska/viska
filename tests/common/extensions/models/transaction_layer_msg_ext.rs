use crate::common::extensions::TryClone;
use common::rsip;
use models::{
    transaction::TransactionLayerMsg,
    transport::{RequestMsg, ResponseMsg, TransportMsg},
};

pub trait TransactionLayerMsgExt {
    fn new_uac_invite_msg(&self) -> RequestMsg;
    fn new_uac_invite_sip_msg(&self) -> rsip::Request {
        self.new_uac_invite_msg().sip_request
    }
    fn new_uas_invite_msg(&self) -> RequestMsg;
    fn new_uas_invite_sip_msg(&self) -> rsip::Request {
        self.new_uas_invite_msg().sip_request
    }
    fn new_uac_msg(&self) -> RequestMsg;
    fn new_uac_sip_msg(&self) -> rsip::Request {
        self.new_uac_msg().sip_request
    }
    //fn new_uas_msg(&self) -> ResponseMsg;
    //fn new_uas_sip_msg(&self) -> rsip::Response {
    //    self.new_uas_msg().sip_response
    //}
    fn reply_msg(&self) -> ResponseMsg;
    fn reply_sip_msg(&self) -> rsip::Response {
        self.reply_msg().sip_response
    }
    fn incoming_msg(&self) -> TransportMsg;
}

impl TransactionLayerMsgExt for TransactionLayerMsg {
    fn new_uac_invite_msg(&self) -> RequestMsg {
        match self {
            TransactionLayerMsg::NewUacInvite(request_msg) => request_msg.clone(),
            _ => panic!("not a NewUacInvite variant"),
        }
    }

    fn new_uas_invite_msg(&self) -> RequestMsg {
        match self {
            TransactionLayerMsg::NewUasInvite(request_msg, _) => request_msg.clone(),
            _ => panic!("not a NewUasInvite variant"),
        }
    }

    fn new_uac_msg(&self) -> RequestMsg {
        match self {
            TransactionLayerMsg::NewUac(request_msg) => request_msg.clone(),
            _ => panic!("not a NewUacInvite variant"),
        }
    }

    //fn new_uas_msg(&self) -> ResponseMsg {
    //    match self {
    //        TransactionLayerMsg::NewUas(request_msg, _) => request_msg.clone(),
    //        _ => panic!("not a NewUasInvite variant"),
    //    }
    //}

    fn reply_msg(&self) -> ResponseMsg {
        match self {
            TransactionLayerMsg::Reply(response_msg) => response_msg.clone(),
            _ => panic!("not a Reply variant"),
        }
    }

    fn incoming_msg(&self) -> TransportMsg {
        match self {
            TransactionLayerMsg::Incoming(transport_msg) => transport_msg.clone(),
            _ => panic!("not an Incoming variant"),
        }
    }
}

impl TryClone for TransactionLayerMsg {
    type Error = String;
    fn try_clone(&self) -> Result<Self, Self::Error> {
        match self {
            TransactionLayerMsg::NewUacInvite(request_msg) => {
                Ok(Self::NewUacInvite(request_msg.clone()))
            }
            TransactionLayerMsg::NewUasInvite(request_msg, opt_response) => Ok(Self::NewUasInvite(
                request_msg.clone(),
                opt_response.clone(),
            )),
            TransactionLayerMsg::NewUac(request_msg) => Ok(Self::NewUac(request_msg.clone())),
            TransactionLayerMsg::NewUas(request_msg, opt_response) => {
                Ok(Self::NewUas(request_msg.clone(), opt_response.clone()))
            }
            TransactionLayerMsg::Reply(response_msg) => Ok(Self::Reply(response_msg.clone())),
            TransactionLayerMsg::Incoming(transport_msg) => {
                Ok(Self::Incoming(transport_msg.clone()))
            }
            TransactionLayerMsg::TransportError(transport_msg, transport_error) => Ok(
                Self::TransportError(transport_msg.clone(), transport_error.clone()),
            ),
            TransactionLayerMsg::HasTransaction(_, _) => {
                Err("can't clone HasTransaction variant, due to Sender".into())
            }
        }
    }
}
