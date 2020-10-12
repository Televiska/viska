use common::libsip::{self};

#[derive(Debug)]
pub struct ContentLength(pub u32);

impl Default for ContentLength {
    fn default() -> Self {
        ContentLength(0)
    }
}

impl Into<u32> for ContentLength {
    fn into(self) -> u32 {
        self.0
    }
}

impl Into<libsip::headers::Header> for ContentLength {
    fn into(self) -> libsip::headers::Header {
        libsip::headers::Header::ContentLength(self.into())
    }
}

impl From<u32> for ContentLength {
    fn from(from: u32) -> Self {
        Self(from)
    }
}
