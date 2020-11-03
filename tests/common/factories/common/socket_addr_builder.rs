use crate::common::factories::prelude::*;
use std::net::{IpAddr, SocketAddr, SocketAddrV4, SocketAddrV6};

#[derive(Debug, Clone)]
pub struct SocketAddrBuilder {
    pub ip_addr: IpAddr,
    pub port: u16,
}

impl SocketAddrBuilder {
    pub fn localhost_with_port(port: u16) -> Self {
        Self {
            ip_addr: IpAddrBuilder::localhost(),
            port,
        }
    }
}

impl Default for SocketAddrBuilder {
    fn default() -> Self {
        Self {
            ip_addr: IpAddrBuilder::default().build(),
            port: 5060,
        }
    }
}

impl Into<SocketAddr> for SocketAddrBuilder {
    fn into(self) -> SocketAddr {
        match self.ip_addr {
            IpAddr::V4(ipv4_addr) => SocketAddr::V4(SocketAddrV4::new(ipv4_addr, self.port)),
            IpAddr::V6(ipv6_addr) => SocketAddr::V6(SocketAddrV6::new(ipv6_addr, self.port, 0, 0)),
        }
    }
}

impl From<(IpAddr, u16)> for SocketAddrBuilder {
    fn from(tuple: (IpAddr, u16)) -> Self {
        Self {
            ip_addr: tuple.0,
            port: tuple.1,
        }
    }
}
