use common::libsip::{self};

#[derive(Debug, Clone)]
pub enum Schema {
    Sip,
    Sips,
}

impl Default for Schema {
    fn default() -> Self {
        Self::Sip
    }
}

impl Into<libsip::UriSchema> for Schema {
    fn into(self) -> libsip::UriSchema {
        match self {
            Self::Sip => libsip::UriSchema::Sip,
            Self::Sips => libsip::UriSchema::Sips,
        }
    }
}
