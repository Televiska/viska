use common::libsip::{
    core::{method::Method, version::Version},
    headers::{via::ViaHeader, ContactHeader, Header, Headers, NamedHeader},
    MissingContactExpiresError, MissingHeaderError, MissingTagError, MissingUsernameError,
    MissingViaBranchError, SipMessage,
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

    pub fn user_agent(&self) -> Result<&String, MissingHeaderError> {
        header!(
            self.headers.0.iter(),
            Header::UserAgent,
            MissingHeaderError::Contact
        )
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
