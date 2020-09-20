use common::{
    bytes::Bytes,
    delegate::delegate,
    libsip::{
        core::{method::Method, version::Version, SipMessageExt},
        header,
        headers::{via::ViaHeader, ContactHeader, Header, Headers, NamedHeader},
        parse_message,
        uri::domain::Domain,
        SipMessage, SipMessageError,
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

    pub fn debug_compact(&self) -> String {
        format!(
            "(Request) {}, {}, {}, {} ",
            self.method()
                .map(|h| h.to_string())
                .unwrap_or_else(|| "".into()),
            self.from_header_domain()
                .map(|h| h.to_string())
                .unwrap_or_else(|_| "".into()),
            self.to_header_domain()
                .map(|h| h.to_string())
                .unwrap_or_else(|_| "".into()),
            self.via_header()
                .map(|h| h.to_string())
                .unwrap_or_else(|_| "".into())
        )
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

    pub fn from_header_domain(&self) -> Result<&Domain, SipMessageError> {
        if let Ok(header) = self.from_header() {
            Ok(&header.uri.host)
        } else {
            Err(SipMessageError::MissingFromUsername)
        }
    }

    pub fn to_header_domain(&self) -> Result<&Domain, SipMessageError> {
        if let Ok(header) = self.to_header() {
            Ok(&header.uri.host)
        } else {
            Err(SipMessageError::MissingFromUsername)
        }
    }

    pub fn contact_header_instance(&self) -> Result<&String, SipMessageError> {
        named_header_param!(
            self.contact_header(),
            "+sip.instance",
            SipMessageError::MissingContactHeader
        )
        .map(|instance| match instance {
            common::libsip::headers::GenValue::Token(inner) => inner,
            common::libsip::headers::GenValue::QuotedString(inner) => inner,
        })
    }

    pub fn user_agent(&self) -> Result<&String, SipMessageError> {
        header!(
            self.inner.headers().0.iter(),
            Header::UserAgent,
            SipMessageError::MissingContactHeader
        )
    }

    delegate! {
        to self.inner {
            pub fn body(&self) -> &Vec<u8>;
            pub fn body_mut(&mut self) -> &mut Vec<u8>;
            pub fn headers(&self) -> &Headers;
            pub fn headers_mut(&mut self) -> &mut Headers;
            pub fn from_header(&self) -> Result<&NamedHeader, SipMessageError>;
            pub fn from_header_mut(&mut self) -> Result<&mut NamedHeader, SipMessageError>;
            pub fn from_header_tag(&self) -> Result<&String, SipMessageError>;
            pub fn set_from_header_tag(&mut self, tag: String);
            pub fn from_header_username(&self) -> Result<&String, SipMessageError>;
            pub fn to_header(&self) -> Result<&NamedHeader, SipMessageError>;
            #[allow(clippy::wrong_self_convention)]
            pub fn to_header_mut(&mut self) -> Result<&mut NamedHeader, SipMessageError>;
            pub fn to_header_tag(&self) -> Result<&String, SipMessageError>;
            pub fn set_to_header_tag(&mut self, tag: String);
            pub fn to_header_username(&self) -> Result<&String, SipMessageError>;
            pub fn via_header(&self) -> Result<&ViaHeader, SipMessageError>;
            pub fn via_header_mut(&mut self) -> Result<&mut ViaHeader, SipMessageError>;
            pub fn via_header_branch(&self) -> Result<&String, SipMessageError>;
            pub fn call_id(&self) -> Result<&String, SipMessageError>;
            pub fn call_id_mut(&mut self) -> Result<&mut String, SipMessageError>;
            pub fn cseq(&self) -> Result<(u32, Method), SipMessageError>;
            pub fn cseq_mut(&mut self) -> Result<(&mut u32, &mut Method), SipMessageError>;
            pub fn contact_header(&self) -> Result<&ContactHeader, SipMessageError>;
            pub fn contact_header_mut(&mut self) -> Result<&mut ContactHeader, SipMessageError>;
            pub fn contact_header_username(&self) -> Result<&String, SipMessageError>;
            pub fn contact_header_expires(&self) -> Result<u32, SipMessageError>;
            pub fn expires_header(&self) -> Result<u32, SipMessageError>;
            pub fn expires_header_mut(&mut self) -> Result<&mut u32, SipMessageError>;
        }
    }
}

impl TryFrom<SipMessage> for Response {
    type Error = &'static str;

    fn try_from(sip_message: SipMessage) -> Result<Self, Self::Error> {
        match sip_message {
            SipMessage::Request { .. } => {
                panic!("Can't convert a SipMessage::Response into Request !")
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
