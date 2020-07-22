use common::{
    libsip::{
        core::{method::Method, version::Version},
        headers::{via::ViaHeader, Header, Headers, NamedHeader},
        uri::{params::UriParam, Uri},
        SipMessage,
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
            (Some(call_id), Some(from_tag), Some(to_tag)) => {
                Some(format!("{}-{}-{}", call_id, from_tag, to_tag))
            }
            _ => None,
        }
    }
}

impl Request {
    pub fn from_header(&self) -> Option<&NamedHeader> {
        header!(self.headers.0.iter(), Header::From)
    }

    pub fn from_header_tag(&self) -> Option<&String> {
        named_header_param!(self.from_header(), "tag")
    }

    pub fn from_header_username(&self) -> Option<&String> {
        self.from_header()
            .and_then(|header| header.uri.auth.as_ref().map(|auth| &auth.username))
    }

    pub fn to_header(&self) -> Option<&NamedHeader> {
        header!(self.headers.0.iter(), Header::To)
    }

    pub fn to_header_tag(&self) -> Option<&String> {
        named_header_param!(self.to_header(), "tag")
    }

    pub fn to_header_username(&self) -> Option<&String> {
        self.to_header()
            .and_then(|header| header.uri.auth.as_ref().map(|auth| &auth.username))
    }

    pub fn via_header(&self) -> Option<&ViaHeader> {
        header!(self.headers.0.iter(), Header::Via)
    }

    pub fn via_header_branch(&self) -> Option<&String> {
        self.via_header().and_then(|header| {
            header.uri.parameters.iter().find_map(|param| {
                if let UriParam::Branch(branch) = param {
                    Some(branch)
                } else {
                    None
                }
            })
        })
    }

    pub fn call_id(&self) -> Option<&String> {
        header!(self.headers.0.iter(), Header::CallId)
    }

    pub fn cseq(&self) -> Option<(u32, Method)> {
        self.headers.0.iter().find_map(|header| {
            if let Header::CSeq(cseq, method) = header {
                Some((*cseq, *method))
            } else {
                None
            }
        })
    }

    pub fn contact_header(&self) -> Option<&NamedHeader> {
        header!(self.headers.0.iter(), Header::Contact)
    }

    pub fn contact_header_expires(&self) -> Option<u32> {
        // https://tools.ietf.org/html/rfc3261#page-228 "c-p-expires" defines that it must be unsigned number
        named_header_param!(self.contact_header(), "expires")
            .and_then(|expires| expires.parse::<u32>().ok())
    }

    pub fn expires_header(&self) -> Option<u32> {
        header!(self.headers.0.iter(), Header::Expires).map(Clone::clone)
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
            Some(call_id) => call_id,
            None => return Err("missing call id".into()),
        }
        .clone();

        let from_tag = match self.from_header_tag() {
            Some(from_header_tag) => from_header_tag,
            None => return Err("missing from header tag".into()),
        }
        .clone();

        let branch_id = match self.via_header_branch() {
            Some(branch_id) => branch_id,
            None => return Err("missing branch id".into()),
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

fn computed_id_for(call_id: &String, from_tag: &String, to_tag: &Uuid) -> String {
    format!("{}-{}-{}", call_id, from_tag, to_tag)
}
