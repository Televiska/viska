use crate::common::factories::RandomizedBuilder;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

#[derive(Debug, Clone)]
pub struct IpAddrBuilder {
    pub version: IpVersion,
    pub multicast: bool,
}

impl Default for IpAddrBuilder {
    fn default() -> Self {
        Self {
            version: Default::default(),
            multicast: false,
        }
    }
}

impl IpAddrBuilder {
    pub fn localhost() -> IpAddr {
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))
    }
}

impl RandomizedBuilder for IpAddrBuilder {
    type Item = IpAddr;

    fn build(self) -> Self::Item {
        use common::rand::{thread_rng, Rng};
        use fake::faker::internet::raw::*;
        use fake::locales::EN;
        use fake::Fake;

        match self.version {
            IpVersion::V4 => {
                if self.multicast {
                    let mut rng = thread_rng();
                    Self::Item::V4(Ipv4Addr::new(224, 0, 0, rng.gen_range(1, 254)))
                } else {
                    Self::Item::V4(IPv4(EN).fake())
                }
            }
            IpVersion::V6 => {
                if self.multicast {
                    let mut rng = thread_rng();
                    Self::Item::V6(Ipv6Addr::new(0xff00, 0, 0, 0, 0, 0, 0, 0x3))
                } else {
                    Self::Item::V6(IPv6(EN).fake())
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum IpVersion {
    V4,
    V6,
}

impl Default for IpVersion {
    fn default() -> Self {
        Self::V4
    }
}
