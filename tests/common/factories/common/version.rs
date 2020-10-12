use common::libsip::{self};

#[derive(Debug, Clone)]
pub enum Version {
    V1,
    V2,
    //Custom(String)
}

impl Default for Version {
    fn default() -> Self {
        Self::V2
    }
}

impl Into<libsip::Version> for Version {
    fn into(self) -> libsip::Version {
        match self {
            Self::V1 => libsip::Version::new(1, 0),
            Self::V2 => libsip::Version::new(2, 0),
        }
    }
}
