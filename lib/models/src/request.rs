use common::libsip::{
    core::{method::Method, version::Version},
    headers::{via::ViaHeader, ContactHeader, Header, Headers, NamedHeader},
    uri::{domain::Domain, Uri},
    MissingContactExpiresError, MissingHeaderError, MissingTagError, MissingUsernameError,
    MissingViaBranchError, SipMessage,
};
use std::convert::TryFrom;

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

    pub fn from_header_domain(&self) -> Result<&Domain, MissingUsernameError> {
        if let Ok(header) = self.from_header() {
            Ok(&header.uri.host)
        } else {
            Err(MissingUsernameError::From)
        }
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

    pub fn contact_header_username(&self) -> Result<&String, MissingUsernameError> {
        if let Ok(header) = self.contact_header() {
            if let Some(auth) = &header.uri.auth {
                Ok(&auth.username)
            } else {
                Err(MissingUsernameError::Contact)
            }
        } else {
            Err(MissingUsernameError::Contact)
        }
    }

    pub fn contact_header_domain(&self) -> Result<&Domain, MissingUsernameError> {
        if let Ok(header) = self.contact_header() {
            Ok(&header.uri.host)
        } else {
            Err(MissingUsernameError::Contact)
        }
    }

    pub fn contact_header_instance(&self) -> Result<&String, MissingHeaderError> {
        named_header_param!(
            self.contact_header(),
            "+sip.instance",
            MissingHeaderError::Contact
        )
        .map(|instance| match instance {
            common::libsip::headers::GenValue::Token(inner) => inner,
            common::libsip::headers::GenValue::QuotedString(inner) => inner,
        })
    }

    pub fn expires_header(&self) -> Result<u32, MissingHeaderError> {
        header!(
            self.headers.0.iter(),
            Header::Expires,
            MissingHeaderError::Expires
        )
        .map(Clone::clone)
    }

    pub fn uri_username(&self) -> Result<&String, MissingUsernameError> {
        if let Some(auth) = &self.uri.auth {
            Ok(&auth.username)
        } else {
            Err(MissingUsernameError::Uri)
        }
    }

    pub fn user_agent(&self) -> Result<&String, MissingHeaderError> {
        header!(
            self.headers.0.iter(),
            Header::UserAgent,
            MissingHeaderError::Contact
        )
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
