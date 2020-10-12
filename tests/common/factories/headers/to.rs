use crate::common::factories::headers::{NamedHeader, NamedParam};
use common::libsip::{self};

#[derive(Debug, Clone)]
pub struct To(pub NamedHeader<NamedParam>);

impl Into<libsip::headers::Header> for To {
    fn into(self) -> libsip::headers::Header {
        libsip::headers::Header::To(Into::<libsip::headers::NamedHeader>::into(self.0))
    }
}

impl From<NamedHeader<NamedParam>> for To {
    fn from(named: NamedHeader<NamedParam>) -> Self {
        Self(named)
    }
}
