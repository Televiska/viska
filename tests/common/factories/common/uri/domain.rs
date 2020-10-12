use common::libsip::{self};

#[derive(Debug, Clone)]
pub struct Domain {
    pub host: String,
    pub port: Option<u16>,
}

//TODO: host default here should be compiled-configured
impl Default for Domain {
    fn default() -> Self {
        Self {
            host: "example.com".into(),
            port: Some(5060),
        }
    }
}

impl Into<libsip::uri::Domain> for Domain {
    fn into(self) -> libsip::uri::Domain {
        libsip::uri::Domain::Domain(self.host, self.port)
    }
}

impl From<(String, Option<u16>)> for Domain {
    fn from(tuple: (String, Option<u16>)) -> Self {
        Self {
            host: tuple.0,
            port: tuple.1,
        }
    }
}
