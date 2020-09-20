use crate::{Error, Request, Response};
use common::{
    bytes::Bytes,
    libsip,
    libsip::{
        core::{method::Method, version::Version},
        uri::domain::Domain,
        SipMessageError,
    },
    nom::error::VerboseError,
};
use std::convert::{TryFrom, TryInto};

#[derive(Debug, Clone)]
pub enum SipMessage {
    Request(Request),
    Response(Response),
}

impl SipMessage {
    pub fn method(&self) -> Option<Method> {
        match self {
            Self::Request(request) => Some(*request.method()),
            Self::Response(response) => response.method(),
        }
    }

    pub fn version(&self) -> &Version {
        match self {
            Self::Request(request) => request.version(),
            Self::Response(response) => response.version(),
        }
    }

    pub fn from_header_domain(&self) -> Result<&Domain, SipMessageError> {
        match self {
            Self::Request(request) => request.from_header_domain(),
            Self::Response(response) => response.from_header_domain(),
        }
    }

    pub fn debug_compact(&self) -> String {
        match self {
            Self::Request(request) => request.debug_compact(),
            Self::Response(response) => response.debug_compact(),
        }
    }
}

impl TryFrom<libsip::SipMessage> for SipMessage {
    type Error = &'static str;

    fn try_from(sip_message: libsip::SipMessage) -> Result<Self, Self::Error> {
        match sip_message {
            libsip::SipMessage::Request { .. } => Ok(SipMessage::Request(
                TryInto::<Request>::try_into(sip_message)?,
            )),
            libsip::SipMessage::Response { .. } => Ok(SipMessage::Response(
                TryInto::<Response>::try_into(sip_message)?,
            )),
        }
    }
}

impl Into<libsip::SipMessage> for SipMessage {
    fn into(self) -> libsip::SipMessage {
        match self {
            SipMessage::Request(request) => request.into(),
            SipMessage::Response(response) => response.into(),
        }
    }
}

impl TryFrom<Bytes> for SipMessage {
    type Error = Error;

    fn try_from(bytes: Bytes) -> Result<Self, Self::Error> {
        let (_, libsip_sip_message) = libsip::parse_message::<VerboseError<&[u8]>>(&bytes.to_vec())
            .map_err(|e| {
                Error::libsip(format!(
                    "could not convert to models::SipMessage from bytes: {}",
                    e
                ))
            })?;

        Ok(libsip_sip_message.try_into()?)
    }
}

impl Into<Bytes> for SipMessage {
    fn into(self) -> Bytes {
        match self {
            SipMessage::Request(request) => {
                Bytes::from(Into::<libsip::SipMessage>::into(request).to_string())
            }
            SipMessage::Response(response) => {
                Bytes::from(Into::<libsip::SipMessage>::into(response).to_string())
            }
        }
    }
}

impl TryFrom<String> for SipMessage {
    type Error = Error;

    fn try_from(string: String) -> Result<Self, Self::Error> {
        let (_, libsip_sip_message) =
            libsip::parse_message::<VerboseError<&[u8]>>(string.as_bytes()).map_err(|e| {
                Error::libsip(format!(
                    "could not convert to models::SipMessage from string: {}",
                    e
                ))
            })?;

        Ok(libsip_sip_message.try_into()?)
    }
}

impl Into<String> for SipMessage {
    fn into(self) -> String {
        match self {
            SipMessage::Request(request) => Into::<libsip::SipMessage>::into(request).to_string(),
            SipMessage::Response(response) => {
                Into::<libsip::SipMessage>::into(response).to_string()
            }
        }
    }
}

impl TryFrom<Vec<u8>> for SipMessage {
    type Error = Error;

    fn try_from(vec: Vec<u8>) -> Result<Self, Self::Error> {
        let (_, libsip_sip_message) =
            libsip::parse_message::<VerboseError<&[u8]>>(&vec).map_err(|e| {
                Error::libsip(format!(
                    "could not convert to models::SipMessage from vec: {}",
                    e
                ))
            })?;

        Ok(libsip_sip_message.try_into()?)
    }
}

impl Into<Vec<u8>> for SipMessage {
    fn into(self) -> Vec<u8> {
        match self {
            SipMessage::Request(request) => Into::<libsip::SipMessage>::into(request)
                .to_string()
                .into_bytes(),
            SipMessage::Response(response) => Into::<libsip::SipMessage>::into(response)
                .to_string()
                .into_bytes(),
        }
    }
}

impl TryFrom<&str> for SipMessage {
    type Error = Error;

    fn try_from(slice: &str) -> Result<Self, Self::Error> {
        let (_, libsip_sip_message) =
            libsip::parse_message::<VerboseError<&[u8]>>(slice.as_bytes()).map_err(|e| {
                Error::libsip(format!(
                    "could not convert to models::SipMessage from vec: {}",
                    e
                ))
            })?;

        Ok(libsip_sip_message.try_into()?)
    }
}

impl From<Request> for SipMessage {
    fn from(request: Request) -> Self {
        SipMessage::Request(request)
    }
}

impl From<Response> for SipMessage {
    fn from(response: Response) -> Self {
        SipMessage::Response(response)
    }
}
