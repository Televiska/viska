use crate::common::extensions::TryClone;
use common::rsip;
use models::transaction::TransactionLayerMsg;

pub trait TransactionLayerMsgExt {
    fn new_uac_invite_msg(&self) -> rsip::Request;
    fn new_uas_invite_msg(&self) -> rsip::Request;
    fn new_uac_msg(&self) -> rsip::Request;
    //fn new_uas_msg(&self) -> rsip::Response;
    fn reply_msg(&self) -> rsip::Response;
    fn incoming_msg(&self) -> rsip::SipMessage;
}

impl TransactionLayerMsgExt for TransactionLayerMsg {
    fn new_uac_invite_msg(&self) -> rsip::Request {
        match self {
            TransactionLayerMsg::NewUacInvite(request) => request.clone(),
            _ => panic!("not a NewUacInvite variant"),
        }
    }

    fn new_uas_invite_msg(&self) -> rsip::Request {
        match self {
            TransactionLayerMsg::NewUasInvite(request, _) => request.clone(),
            _ => panic!("not a NewUasInvite variant"),
        }
    }

    fn new_uac_msg(&self) -> rsip::Request {
        match self {
            TransactionLayerMsg::NewUac(request) => request.clone(),
            _ => panic!("not a NewUacInvite variant"),
        }
    }

    //fn new_uas_msg(&self) -> rsip::Response {
    //    match self {
    //        TransactionLayerMsg::NewUas(request, _) => request.clone(),
    //        _ => panic!("not a NewUasInvite variant"),
    //    }
    //}

    fn reply_msg(&self) -> rsip::Response {
        match self {
            TransactionLayerMsg::Reply(response) => response.clone(),
            _ => panic!("not a Reply variant"),
        }
    }

    fn incoming_msg(&self) -> rsip::SipMessage {
        match self {
            TransactionLayerMsg::Incoming(msg) => msg.clone(),
            _ => panic!("not an Incoming variant"),
        }
    }
}

impl TryClone for TransactionLayerMsg {
    type Error = String;
    fn try_clone(&self) -> Result<Self, Self::Error> {
        match self {
            TransactionLayerMsg::NewUacInvite(request) => Ok(Self::NewUacInvite(request.clone())),
            TransactionLayerMsg::NewUasInvite(request, opt_response) => {
                Ok(Self::NewUasInvite(request.clone(), opt_response.clone()))
            }
            TransactionLayerMsg::NewUac(request) => Ok(Self::NewUac(request.clone())),
            TransactionLayerMsg::NewUas(request, opt_response) => {
                Ok(Self::NewUas(request.clone(), opt_response.clone()))
            }
            TransactionLayerMsg::Reply(response) => Ok(Self::Reply(response.clone())),
            TransactionLayerMsg::Incoming(msg) => Ok(Self::Incoming(msg.clone())),
            TransactionLayerMsg::TransportError(msg, transport_error) => {
                Ok(Self::TransportError(msg.clone(), transport_error.clone()))
            }
            TransactionLayerMsg::HasTransaction(_, _) => {
                Err("can't clone HasTransaction variant, due to Sender".into())
            }
        }
    }
}
