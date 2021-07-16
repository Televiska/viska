use super::SocketAddrBuilder;
use crate::common::factories::RandomizedBuilder;
use common::rsip::{prelude::*, Auth, Host, HostWithPort, Param, Schema, Uri};
use std::net::SocketAddr;

pub trait UriExt {
    fn with_schema(self, schema: Option<Schema>) -> Self;
    fn sip(self) -> Self;
    fn sips(self) -> Self;
    fn with_auth(self, auth: Option<Auth>) -> Self;
    fn with_username(self, username: impl Into<String>) -> Self;
    fn with_host_with_port(self, host_with_port: impl Into<HostWithPort>) -> Self;
    fn with_host(self, host: impl Into<HostWithPort>) -> Self;
    fn with_port(self, port: u16) -> Self;
    fn with_param(self, param: Param) -> Self;
    fn with_params(self, params: Vec<Param>) -> Self;
    fn stripped(self) -> Self
    where
        Self: Sized,
    {
        self.with_auth(None).with_params(vec![]).with_schema(None)
    }
}

impl UriExt for Uri {
    fn with_schema(mut self, schema: Option<Schema>) -> Self {
        self.schema = schema;
        self
    }
    fn sip(mut self) -> Self {
        self.schema = Some(Schema::Sip);
        self
    }
    fn sips(mut self) -> Self {
        self.schema = Some(Schema::Sips);
        self
    }
    fn with_auth(mut self, auth: Option<Auth>) -> Self {
        self.auth = auth;
        self
    }
    fn with_username(mut self, username: impl Into<String>) -> Self {
        self.auth = Some(Auth {
            username: username.into(),
            password: self.auth.map(|auth| auth.password).flatten(),
        });
        self
    }
    fn with_host_with_port(mut self, host_with_port: impl Into<HostWithPort>) -> Self {
        self.host_with_port = host_with_port.into();
        self
    }
    fn with_host(mut self, host_with_port: impl Into<HostWithPort>) -> Self {
        self.host_with_port = HostWithPort {
            host: host_with_port.into().host,
            port: self.host_with_port.port,
        };
        self
    }
    fn with_port(mut self, port: u16) -> Self {
        self.host_with_port = HostWithPort {
            host: self.host_with_port.host,
            port: Some(port.into()),
        };
        self
    }
    fn with_param(mut self, param: Param) -> Self {
        self.params.push(param);
        self
    }
    fn with_params(mut self, params: Vec<Param>) -> Self {
        self.params = params;
        self
    }
}
pub trait HostWithPortExt {
    fn localhost_with_port(port: u16) -> HostWithPort;
}

impl HostWithPortExt for HostWithPort {
    fn localhost_with_port(port: u16) -> Self {
        Self {
            port: Some(port.into()),
            ..Default::default()
        }
    }
}

/*
impl RandomizedBuilder for HostWithPort {
    type Item = Self;

    fn build(self) -> Self::Item {
    }
}*/
