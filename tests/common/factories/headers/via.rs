use common::libsip::{self};

use crate::common::factories::common::{uri::Uri, Transport, Version};

#[derive(Debug, Clone)]
pub struct Via {
    pub version: Version,
    pub transport: Transport,
    pub uri: Uri,
}

impl Default for Via {
    fn default() -> Self {
        Self {
            version: Default::default(),
            transport: Default::default(),
            uri: Default::default(),
        }
    }
}

impl Into<libsip::headers::via::ViaHeader> for Via {
    fn into(self) -> libsip::headers::via::ViaHeader {
        libsip::headers::via::ViaHeader {
            version: self.version.into(),
            transport: self.transport.into(),
            uri: self.uri.into(),
        }
    }
}

impl Into<libsip::headers::Header> for Via {
    fn into(self) -> libsip::headers::Header {
        libsip::headers::Header::Via(libsip::headers::via::ViaHeader {
            version: self.version.into(),
            transport: self.transport.into(),
            uri: self.uri.into(),
        })
    }
}
