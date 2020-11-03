use super::SocketAddrBuilder;
use crate::common::factories::RandomizedBuilder;
use rsip::common::uri::{HostWithPort, Uri};

pub trait UriExt {
    fn localhost() -> Uri {
        Self::localhost_with_port(5060)
    }
    fn localhost_with_port(port: u16) -> Uri;
}

impl UriExt for Uri {
    fn localhost_with_port(port: u16) -> Uri {
        Uri {
            schema: None,
            host_with_port: HostWithPort::localhost_with_port(port),
            auth: None,
            params: Default::default(),
        }
    }
}

pub trait HostWithPortExt {
    fn localhost_with_port(port: u16) -> HostWithPort;
}

impl HostWithPortExt for HostWithPort {
    fn localhost_with_port(port: u16) -> HostWithPort {
        HostWithPort::SocketAddr(SocketAddrBuilder::localhost_with_port(port).into())
    }
}

/*
impl RandomizedBuilder for HostWithPort {
    type Item = Self;

    fn build(self) -> Self::Item {
    }
}*/
