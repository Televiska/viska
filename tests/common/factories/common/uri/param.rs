use common::libsip::{self};

#[derive(Debug, Clone)]
pub enum Param {
    Transport(super::super::Transport),
    Branch(Branch),
    Received(super::Domain),
    RPort(Option<u16>),
    Other(String, Option<String>),
}

impl Into<libsip::uri::UriParam> for Param {
    fn into(self) -> libsip::uri::UriParam {
        match self {
            Self::Transport(transport) => libsip::uri::UriParam::Transport(transport.into()),
            Self::Branch(branch) => libsip::uri::UriParam::Branch(branch.into()),
            Self::Received(domain) => libsip::uri::UriParam::Received(domain.into()),
            Self::RPort(port) => libsip::uri::UriParam::RPort(port),
            Self::Other(key, value) => libsip::uri::UriParam::Other(key, value),
        }
    }
}

//TODO: should RFC2543 as well
//TODO: add randomized trait in here
#[derive(Debug, Clone)]
pub struct Branch(String);
impl Default for Branch {
    fn default() -> Self {
        Branch("z9hG4bKthisisarandombranch".into())
    }
}

impl Into<String> for Branch {
    fn into(self) -> String {
        self.0
    }
}
