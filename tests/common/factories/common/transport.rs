use common::libsip::{self};

#[derive(Debug, Clone)]
pub enum Transport {
    Udp,
    Tcp,
}

impl Default for Transport {
    fn default() -> Self {
        Self::Udp
    }
}

impl Into<libsip::Transport> for Transport {
    fn into(self) -> libsip::Transport {
        match self {
            Self::Udp => libsip::Transport::Udp,
            Self::Tcp => libsip::Transport::Tcp,
        }
    }
}

impl Into<models::TransportType> for Transport {
    fn into(self) -> models::TransportType {
        match self {
            Self::Udp => models::TransportType::Udp,
            Self::Tcp => models::TransportType::Tcp,
        }
    }
}
