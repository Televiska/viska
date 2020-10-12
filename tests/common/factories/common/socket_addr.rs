use crate::common::factories::common::ip_addr::IpAddr;
use crate::common::factories::RandomizedBuilder;
use common::libsip::{self};
use std::net::{
    IpAddr as StdIpAddr, SocketAddr as StdSocketAddr, SocketAddrV4 as StdSocketAddrV4,
    SocketAddrV6 as StdSocketAddrV6,
};

#[derive(Debug, Clone)]
pub struct SocketAddr {
    pub ip_addr: StdIpAddr,
    pub port: u16,
}

impl Default for SocketAddr {
    fn default() -> Self {
        Self {
            ip_addr: IpAddr::default().build(),
            port: 5060,
        }
    }
}

impl Into<StdSocketAddr> for SocketAddr {
    fn into(self) -> StdSocketAddr {
        match self.ip_addr {
            StdIpAddr::V4(ipv4_addr) => {
                StdSocketAddr::V4(StdSocketAddrV4::new(ipv4_addr, self.port))
            }
            StdIpAddr::V6(ipv6_addr) => {
                StdSocketAddr::V6(StdSocketAddrV6::new(ipv6_addr, self.port, 0, 0))
            }
        }
    }
}

impl Into<libsip::uri::Domain> for SocketAddr {
    fn into(self) -> libsip::uri::Domain {
        let port = self.port;
        match self.ip_addr {
            StdIpAddr::V4(ip_addr) => libsip::uri::Domain::Ipv4(ip_addr, Some(port)),
            StdIpAddr::V6(_) => panic!("libsip does not support V4 ip addr"),
        }
    }
}

impl From<(StdIpAddr, u16)> for SocketAddr {
    fn from(tuple: (StdIpAddr, u16)) -> Self {
        Self {
            ip_addr: tuple.0,
            port: tuple.1,
        }
    }
}

pub trait TestsSocketAddrExt {
    fn localhost_with_port(port: u16) -> SocketAddr;
}

impl TestsSocketAddrExt for SocketAddr {
    fn localhost_with_port(port: u16) -> Self {
        use super::TestsStdIpAddrExt;

        Self {
            ip_addr: StdIpAddr::localhost(),
            port,
        }
    }
}
