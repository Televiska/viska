mod auth;
mod domain;
mod host_with_port;
mod param;
mod schema;

pub use auth::Auth;
pub use domain::Domain;
pub use host_with_port::HostWithPort;
pub use param::{Branch, Param};
pub use schema::Schema;

use common::libsip::{self};

#[derive(Debug, Clone)]
pub struct Uri {
    pub schema: Schema,
    pub host_with_port: HostWithPort,
    pub auth: Option<Auth>,
    pub params: Vec<Param>,
}

pub trait TestsUriExt {
    fn localhost() -> Uri;
    fn localhost_with_port(port: u16) -> Uri;
}

impl TestsUriExt for Uri {
    fn localhost() -> Self {
        use super::TestsStdIpAddrExt;
        use std::net::IpAddr as StdIpAddr;

        Self {
            host_with_port: StdIpAddr::localhost().into(),
            ..Default::default()
        }
    }

    fn localhost_with_port(port: u16) -> Self {
        use crate::common::factories::common::{uri::Domain, SocketAddr, TestsSocketAddrExt};

        Self {
            host_with_port: SocketAddr::localhost_with_port(port).into(),
            ..Default::default()
        }
    }
}

impl Default for Uri {
    fn default() -> Self {
        Self {
            schema: Default::default(),
            host_with_port: Default::default(),
            auth: Default::default(),
            params: Default::default(),
        }
    }
}

impl Into<libsip::uri::Uri> for Uri {
    fn into(self) -> libsip::uri::Uri {
        libsip::uri::Uri {
            schema: Some(self.schema.into()),
            host: self.host_with_port.into(),
            auth: self.auth.map(|a| a.into()),
            parameters: self
                .params
                .into_iter()
                .map(|p| p.into())
                .collect::<Vec<_>>(),
        }
    }
}
