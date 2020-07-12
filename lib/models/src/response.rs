use common::libsip::{
    core::{method::Method, version::Version},
    headers::{via::ViaHeader, Header, Headers, NamedHeader},
    uri::params::UriParam,
    SipMessage,
};
use std::convert::TryFrom;

#[derive(Debug, Clone)]
pub struct Response {
    pub code: u32,
    pub version: Version,
    pub headers: Headers,
    pub body: Vec<u8>,
}

impl Response {
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

impl TryFrom<SipMessage> for Response {
    type Error = &'static str;

    fn try_from(sip_message: SipMessage) -> Result<Self, Self::Error> {
        match sip_message {
            SipMessage::Response {
                code,
                version,
                headers,
                body,
            } => Ok(Self {
                code,
                version,
                headers,
                body,
            }),
            SipMessage::Request { .. } => Err("Can't convert a SipMessage::Request"),
        }
    }
}

impl Into<SipMessage> for Response {
    fn into(self) -> SipMessage {
        SipMessage::Response {
            code: self.code,
            version: self.version,
            headers: self.headers,
            body: self.body,
        }
    }
}
