use crate::common::factories::common::{uri::Domain, SocketAddr};
use common::libsip::{self};
use std::net::IpAddr as StdIpAddr;

#[derive(Debug, Clone)]
pub enum HostWithPort {
    Domain(Domain),
    SocketAddr(SocketAddr),
    IpAddr(StdIpAddr),
}

impl Default for HostWithPort {
    fn default() -> Self {
        Self::SocketAddr(Default::default())
    }
}

impl Into<libsip::uri::Domain> for HostWithPort {
    fn into(self) -> libsip::uri::Domain {
        match self {
            Self::Domain(domain) => domain.into(),
            Self::SocketAddr(socket_addr) => socket_addr.into(),
            Self::IpAddr(ip_addr) => match ip_addr {
                StdIpAddr::V4(ip_addr) => libsip::uri::Domain::Ipv4(ip_addr, None),
                StdIpAddr::V6(_) => panic!("libsip does not support V4 ip addr"),
            },
        }
    }
}

impl From<StdIpAddr> for HostWithPort {
    fn from(std_ip_addr: StdIpAddr) -> Self {
        Self::IpAddr(std_ip_addr)
    }
}

impl From<SocketAddr> for HostWithPort {
    fn from(socket_addr: SocketAddr) -> Self {
        Self::SocketAddr(socket_addr)
    }
}

impl From<Domain> for HostWithPort {
    fn from(domain: Domain) -> Self {
        Self::Domain(domain)
    }
}
