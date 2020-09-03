use common::{
    bytes::Bytes,
    delegate::delegate,
    libsip::{
        core::{method::Method, version::Version, SipMessageExt},
        header,
        headers::{via::ViaHeader, ContactHeader, Header, Headers, NamedHeader},
        parse_message,
        uri::domain::Domain,
        MissingContactExpiresError, MissingHeaderError, MissingTagError, MissingUsernameError,
        MissingViaBranchError, SipMessage,
    },
    nom::error::VerboseError,
};
use std::convert::TryFrom;

#[derive(Debug, Clone)]
pub struct Response {
    inner: SipMessage,
}

impl Response {
    pub fn dialog_id(&self) -> Option<String> {
        match (
            self.inner.call_id(),
            self.inner.from_header_tag(),
            self.inner.to_header_tag(),
        ) {
            (Ok(call_id), Ok(from_tag), Ok(to_tag)) => {
                Some(format!("{}-{}-{}", call_id, from_tag, to_tag))
            }
            _ => None,
        }
    }

    pub fn method(&self) -> Option<Method> {
        match self.inner {
            SipMessage::Request { .. } => panic!(state_mismatch_for("method")),
            SipMessage::Response { .. } => self.cseq().ok().map(|(_, method)| method),
        }
    }

    pub fn status_code(&self) -> u32 {
        match self.inner {
            SipMessage::Request { .. } => panic!(state_mismatch_for("code")),
            SipMessage::Response { code, .. } => code,
        }
    }

    pub fn version(&self) -> &Version {
        match &self.inner {
            SipMessage::Request { version, .. } => version,
            _ => panic!(state_mismatch_for("version")),
        }
    }

    pub fn from_header_domain(&self) -> Result<&Domain, MissingUsernameError> {
        if let Ok(header) = self.from_header() {
            Ok(&header.uri.host)
        } else {
            Err(MissingUsernameError::From)
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

    pub fn user_agent(&self) -> Result<&String, MissingHeaderError> {
        header!(
            self.inner.headers().0.iter(),
            Header::UserAgent,
            MissingHeaderError::Contact
        )
    }

    delegate! {
        to self.inner {
            pub fn body(&self) -> &Vec<u8>;
            pub fn body_mut(&mut self) -> &mut Vec<u8>;
            pub fn headers(&self) -> &Headers;
            pub fn headers_mut(&mut self) -> &mut Headers;
            pub fn from_header(&self) -> Result<&NamedHeader, MissingHeaderError>;
            pub fn from_header_mut(&mut self) -> Result<&mut NamedHeader, MissingHeaderError>;
            pub fn from_header_tag(&self) -> Result<&String, MissingTagError>;
            pub fn set_from_header_tag(&mut self, tag: String);
            pub fn from_header_username(&self) -> Result<&String, MissingUsernameError>;
            pub fn to_header(&self) -> Result<&NamedHeader, MissingHeaderError>;
            #[allow(clippy::wrong_self_convention)]
            pub fn to_header_mut(&mut self) -> Result<&mut NamedHeader, MissingHeaderError>;
            pub fn to_header_tag(&self) -> Result<&String, MissingTagError>;
            pub fn set_to_header_tag(&mut self, tag: String);
            pub fn to_header_username(&self) -> Result<&String, MissingUsernameError>;
            pub fn via_header(&self) -> Result<&ViaHeader, MissingHeaderError>;
            pub fn via_header_mut(&mut self) -> Result<&mut ViaHeader, MissingHeaderError>;
            pub fn via_header_branch(&self) -> Result<&String, MissingViaBranchError>;
            pub fn call_id(&self) -> Result<&String, MissingHeaderError>;
            pub fn call_id_mut(&mut self) -> Result<&mut String, MissingHeaderError>;
            pub fn cseq(&self) -> Result<(u32, Method), MissingHeaderError>;
            pub fn cseq_mut(&mut self) -> Result<(&mut u32, &mut Method), MissingHeaderError>;
            pub fn contact_header(&self) -> Result<&ContactHeader, MissingHeaderError>;
            pub fn contact_header_mut(&mut self) -> Result<&mut ContactHeader, MissingHeaderError>;
            pub fn contact_header_username(&self) -> Result<&String, MissingUsernameError>;
            pub fn contact_header_expires(&self) -> Result<u32, MissingContactExpiresError>;
            pub fn expires_header(&self) -> Result<u32, MissingHeaderError>;
            pub fn expires_header_mut(&mut self) -> Result<&mut u32, MissingHeaderError>;
        }
    }
}

impl TryFrom<SipMessage> for Response {
    type Error = &'static str;

    fn try_from(sip_message: SipMessage) -> Result<Self, Self::Error> {
        match sip_message {
            SipMessage::Request { .. } => {
                Err("Can't convert a SipMessage::Request into Response !")
            }
            SipMessage::Response { .. } => Ok(Self { inner: sip_message }),
        }
    }
}

impl Into<SipMessage> for Response {
    fn into(self) -> SipMessage {
        self.inner
    }
}

impl TryFrom<Bytes> for Response {
    type Error = String;

    fn try_from(bytes: Bytes) -> Result<Self, Self::Error> {
        use std::convert::TryInto;

        let (_, sip_message) =
            parse_message::<VerboseError<&[u8]>>(&bytes.to_vec()).map_err(|e| e.to_string())?;

        Ok(sip_message.try_into()?)
    }
}

fn state_mismatch_for(part: &str) -> String {
    format!("SipMessage and Response mismatch: can't fetch {}", part)
}
