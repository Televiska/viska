use common::{
    libsip::{
        core::{method::Method, version::Version},
        headers::{via::ViaHeader, ContactHeader, Header, Headers, NamedHeader},
        uri::Uri,
        MissingContactExpiresError, MissingHeaderError, MissingTagError, MissingUsernameError,
        MissingViaBranchError, SipMessage,
    },
    uuid::Uuid,
};
use std::convert::{TryFrom, TryInto};

#[derive(Debug, Clone)]
pub struct Request {
    pub method: Method,
    pub uri: Uri,
    pub version: Version,
    pub headers: Headers,
    pub body: Vec<u8>,
}

impl Request {
    pub fn dialog_id(&self) -> Option<String> {
        match (self.call_id(), self.from_header_tag(), self.to_header_tag()) {
            (Ok(call_id), Ok(from_tag), Ok(to_tag)) => {
                Some(format!("{}-{}-{}", call_id, from_tag, to_tag))
            }
            _ => None,
        }
    }
}

impl Request {
    pub fn from_header(&self) -> Result<&NamedHeader, MissingHeaderError> {
        header!(
            self.headers.0.iter(),
            Header::From,
            MissingHeaderError::From
        )
    }

    pub fn from_header_tag(&self) -> Result<&String, MissingTagError> {
        named_header_param!(self.from_header(), "tag", MissingTagError::From)
    }

    pub fn from_header_username(&self) -> Result<&String, MissingUsernameError> {
        named_header_username!(self.from_header(), MissingUsernameError::From)
    }

    pub fn to_header(&self) -> Result<&NamedHeader, MissingHeaderError> {
        header!(self.headers.0.iter(), Header::To, MissingHeaderError::To)
    }

    pub fn to_header_tag(&self) -> Result<&String, MissingTagError> {
        named_header_param!(self.to_header(), "tag", MissingTagError::To)
    }

    pub fn to_header_username(&self) -> Result<&String, MissingUsernameError> {
        named_header_username!(self.to_header(), MissingUsernameError::To)
    }

    pub fn via_header(&self) -> Result<&ViaHeader, MissingHeaderError> {
        header!(self.headers.0.iter(), Header::Via, MissingHeaderError::Via)
    }

    pub fn via_header_branch(&self) -> Result<&String, MissingViaBranchError> {
        if let Ok(header) = self.via_header() {
            header.branch().ok_or(MissingViaBranchError)
        } else {
            Err(MissingViaBranchError)
        }
    }

    pub fn call_id(&self) -> Result<&String, MissingHeaderError> {
        header!(
            self.headers.0.iter(),
            Header::CallId,
            MissingHeaderError::CallId
        )
    }

    pub fn cseq(&self) -> Result<(u32, Method), MissingHeaderError> {
        self.headers
            .0
            .iter()
            .find_map(|header| {
                if let Header::CSeq(cseq, method) = header {
                    Some((*cseq, *method))
                } else {
                    None
                }
            })
            .ok_or(MissingHeaderError::CSeq)
    }

    pub fn contact_header(&self) -> Result<&ContactHeader, MissingHeaderError> {
        header!(
            self.headers.0.iter(),
            Header::Contact,
            MissingHeaderError::Contact
        )
    }

    pub fn contact_header_expires(&self) -> Result<u32, MissingContactExpiresError> {
        // https://tools.ietf.org/html/rfc3261#page-228 "c-p-expires" defines that it must be unsigned number
        named_header_param!(self.contact_header(), "expires", MissingContactExpiresError).and_then(
            |expires| {
                expires
                    .to_string()
                    .parse::<u32>()
                    .map_err(|_| MissingContactExpiresError)
            },
        )
    }

    pub fn expires_header(&self) -> Result<u32, MissingHeaderError> {
        header!(
            self.headers.0.iter(),
            Header::Expires,
            MissingHeaderError::Expires
        )
        .map(Clone::clone)
    }
}

impl Into<store::DirtyRequest> for Request {
    fn into(self) -> store::DirtyRequest {
        store::DirtyRequest {
            method: Some(self.method.to_string()),
            uri: Some(self.uri.to_string()),
            headers: Some(format!("{:?}", self.headers)),
            body: Some(String::from_utf8_lossy(&self.body).to_string()),
            ..Default::default()
        }
    }
}

impl TryFrom<SipMessage> for Request {
    type Error = &'static str;

    fn try_from(sip_message: SipMessage) -> Result<Self, Self::Error> {
        match sip_message {
            SipMessage::Request {
                method,
                uri,
                version,
                headers,
                body,
            } => Ok(Self {
                method,
                uri,
                version,
                headers,
                body,
            }),
            SipMessage::Response { .. } => Err("Can't convert a SipMessage::Response"),
        }
    }
}

impl Into<SipMessage> for Request {
    fn into(self) -> SipMessage {
        SipMessage::Request {
            method: self.method,
            uri: self.uri,
            version: self.version,
            headers: self.headers,
            body: self.body,
        }
    }
}

impl TryInto<store::DirtyDialogWithTransaction> for Request {
    type Error = String;

    fn try_into(self) -> Result<store::DirtyDialogWithTransaction, Self::Error> {
        //use store::{DialogFlow, RegistrationFlow};
        let call_id = match self.call_id() {
            Ok(call_id) => call_id,
            Err(_) => return Err("missing call id".into()),
        }
        .clone();

        let from_tag = match self.from_header_tag() {
            Ok(from_header_tag) => from_header_tag,
            Err(_) => return Err("missing from header tag".into()),
        }
        .clone();

        let branch_id = match self.via_header_branch() {
            Ok(branch_id) => branch_id,
            Err(_) => return Err("missing branch id".into()),
        }
        .clone();

        let to_tag = Uuid::new_v4();

        Ok(store::DirtyDialogWithTransaction {
            dialog: store::DirtyDialog {
                computed_id: Some(computed_id_for(&call_id, &from_tag, &to_tag)),
                call_id: Some(call_id),
                from_tag: Some(from_tag),
                to_tag: Some(to_tag.to_string()),
                flow: Some(flow_for_method(self.method)?),
                ..Default::default()
            },
            transaction: store::DirtyTransaction {
                branch_id: Some(branch_id),
                state: Some(store::TransactionState::Trying),
                ..Default::default()
            },
        })
    }
}

//this doesn't really fit in here
fn flow_for_method(method: Method) -> Result<store::DialogFlow, String> {
    match method {
        Method::Register => Ok(store::DialogFlow::Registration),
        Method::Invite => Ok(store::DialogFlow::Invite),
        Method::Publish => Ok(store::DialogFlow::Publish),
        Method::Subscribe => Ok(store::DialogFlow::Publish),
        _ => Err("Unsupported method".into()),
    }
}

fn computed_id_for(call_id: &str, from_tag: &str, to_tag: &Uuid) -> String {
    format!("{}-{}-{}", call_id, from_tag, to_tag)
}
