use crate::common::factories::headers::{ContactParam, NamedHeader};
use common::libsip::{self};

#[derive(Debug, Clone)]
pub struct Contact(pub NamedHeader<ContactParam>);

impl Into<libsip::headers::ContactHeader> for Contact {
    fn into(self) -> libsip::headers::ContactHeader {
        libsip::headers::ContactHeader {
            display_name: self.0.display_name,
            uri: self.0.uri.into(),
            parameters: self.0.params.into(),
        }
    }
}

impl Into<libsip::headers::Header> for Contact {
    fn into(self) -> libsip::headers::Header {
        libsip::headers::Header::Contact(Into::<libsip::headers::ContactHeader>::into(self))
    }
}

impl From<NamedHeader<ContactParam>> for Contact {
    fn from(named: NamedHeader<ContactParam>) -> Self {
        Self(named)
    }
}
