use crate::common::factories::headers::{NamedHeader, NamedParam};
use common::libsip::{self};

#[derive(Debug, Clone)]
pub struct From(pub NamedHeader<NamedParam>);

impl Into<libsip::headers::Header> for From {
    fn into(self) -> libsip::headers::Header {
        libsip::headers::Header::From(Into::<libsip::headers::NamedHeader>::into(self.0))
    }
}

impl std::convert::From<NamedHeader<NamedParam>> for From {
    fn from(named: NamedHeader<NamedParam>) -> Self {
        Self(named)
    }
}
