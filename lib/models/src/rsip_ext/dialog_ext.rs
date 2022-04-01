use crate::tu::DialogId;
use crate::Error;
use common::rsip::{self, headers::ToTypedHeader, message::HeadersExt};

pub trait DialogExt {
    fn dialog_id(&self) -> Result<DialogId, Error>;
}

impl DialogExt for rsip::Request {
    fn dialog_id(&self) -> Result<DialogId, Error> {
        let call_id = self.call_id_header()?;
        let local_tag = self
            .from_header()?
            .typed()?
            .tag()
            .ok_or_else(|| Error::from("missing from tag"))?
            .clone();
        let remote_tag = self.to_header()?.typed()?.tag().cloned();

        Ok(DialogId::new(call_id, local_tag, remote_tag))
    }
}

impl DialogExt for rsip::Response {
    fn dialog_id(&self) -> Result<DialogId, Error> {
        let call_id = self.call_id_header()?;
        let local_tag = self
            .from_header()?
            .typed()?
            .tag()
            .ok_or_else(|| Error::from("missing from tag"))?
            .clone();
        let remote_tag = self.to_header()?.typed()?.tag().cloned();

        Ok(DialogId::new(call_id, local_tag, remote_tag))
    }
}

impl DialogExt for rsip::SipMessage {
    fn dialog_id(&self) -> Result<DialogId, Error> {
        match self {
            Self::Request(request) => request.dialog_id(),
            Self::Response(response) => response.dialog_id(),
        }
    }
}
