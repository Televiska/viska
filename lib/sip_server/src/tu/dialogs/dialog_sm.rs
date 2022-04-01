//use super::{uac, uas};
use super::uac;
//use models::transport::ResponseMsg;
use crate::Error;
use common::rsip;

#[derive(Debug)]
pub enum DialogSm {
    Uac(uac::MultiDialog),
    //Uas(uas::MultiDialog),
}

impl DialogSm {
    pub async fn process_incoming_request(&self, msg: rsip::Request) -> Result<(), Error> {
        match self {
            Self::Uac(uac) => uac.process_incoming_request(msg).await,
            //Self::Uas(uas) => uas.process_response(msg).await?,
        }
    }

    pub async fn process_incoming_response(&self, msg: rsip::Response) -> Result<(), Error> {
        match self {
            Self::Uac(uac) => uac.process_incoming_response(msg).await,
            //Self::Uas(uas) => uas.process_response(msg).await?,
        }
    }

    pub async fn transport_error(&self, reason: String, msg: rsip::SipMessage) {
        match self {
            Self::Uac(uac) => uac.transport_error(reason, msg).await,
            //Self::Uas(uas) => uas.process_response(msg).await?,
        }
    }
}

impl From<uac::MultiDialog> for DialogSm {
    fn from(from: uac::MultiDialog) -> Self {
        Self::Uac(from)
    }
}
/*
impl From<uas::MultiDialog> for DialogSm {
    fn from(from: uas::MultiDialog) -> Self {
        Self::Uas(from)
    }
}*/
