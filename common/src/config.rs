use envconfig::Envconfig;
use std::net::{IpAddr, SocketAddr};

#[derive(envconfig::Envconfig, Debug, Clone)]
pub struct EnvConfig {
    #[envconfig(from = "DATABASE_URL")]
    pub database_url: String,
}

#[allow(clippy::new_without_default)]
impl EnvConfig {
    pub fn new() -> Self {
        Self::init_from_env().expect("failed to read config from env")
    }
}

//TODO: add port config
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub ip_addrs: Vec<IpAddr>,
    pub default_ip_addr: IpAddr,
}

impl Config {
    pub fn new() -> Self {
        let env_config = EnvConfig::new();
        let ip_addrs = pnet::datalink::interfaces()
            .into_iter()
            .map(|i| i.ips)
            .flatten()
            .map(|net| net.ip())
            .collect::<Vec<IpAddr>>();

        Self {
            database_url: env_config.database_url,
            default_ip_addr: ip_addrs
                .clone()
                .into_iter()
                .find(|ip| ip.is_ipv4() && !ip.is_loopback() && !ip.is_multicast())
                .expect("cannot bind default ip address"),
            ip_addrs,
        }
    }

    pub fn default_socket_addr(&self) -> SocketAddr {
        SocketAddr::new(self.default_ip_addr, 5060)
    }
}
