use crate::common::factories::RandomizedBuilder;
use std::net::{IpAddr as StdIpAddr, Ipv4Addr as StdIpv4Addr, Ipv6Addr as StdIpv6Addr};

//TODO: This IpAddr model is probably not needed, instead we should derive a TestsExt with
//necessary methods

#[derive(Debug, Clone)]
pub struct IpAddr {
    pub version: IpVersion,
    pub multicast: bool,
}

impl Default for IpAddr {
    fn default() -> Self {
        Self {
            version: Default::default(),
            multicast: false,
        }
    }
}

pub trait TestsStdIpAddrExt {
    fn localhost() -> StdIpAddr;
}

impl TestsStdIpAddrExt for StdIpAddr {
    fn localhost() -> Self {
        Self::V4(StdIpv4Addr::new(127, 0, 0, 1))
    }
}

//TODO: this should be RandomizedBuilder trait!
impl RandomizedBuilder for IpAddr {
    type Item = StdIpAddr;

    fn build(self) -> Self::Item {
        use common::rand::{thread_rng, Rng};
        use fake::faker::internet::raw::*;
        use fake::locales::EN;
        use fake::Fake;

        match self.version {
            IpVersion::V4 => {
                if self.multicast {
                    let mut rng = thread_rng();
                    Self::Item::V4(StdIpv4Addr::new(224, 0, 0, rng.gen_range(1, 254)))
                } else {
                    Self::Item::V4(IPv4(EN).fake())
                }
            }
            IpVersion::V6 => {
                if self.multicast {
                    let mut rng = thread_rng();
                    Self::Item::V6(StdIpv6Addr::new(0xff00, 0, 0, 0, 0, 0, 0, 0x3))
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
