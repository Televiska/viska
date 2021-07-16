use envconfig::Envconfig;
use rsip::HostWithPort;
use std::convert::TryInto;
use std::net::IpAddr;

#[derive(envconfig::Envconfig, Debug, Clone)]
pub struct EnvConfig {
    #[envconfig(from = "DATABASE_URL")]
    pub database_url: String,
    #[envconfig(from = "LISTEN_ADDRS")]
    pub listen_addrs: Option<String>,
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
    pub listen_addrs: Vec<HostWithPort>,
    pub default_listen_addr: HostWithPort,
}

impl Default for Config {
    fn default() -> Self {
        let env_config = EnvConfig::new();
        let (default_listen_addr, listen_addrs) = figure_out_listen_addrs(env_config.listen_addrs);

        Self {
            database_url: env_config.database_url,
            listen_addrs,
            default_listen_addr,
        }
    }
}

impl Config {
    pub fn default_addr(&self) -> HostWithPort {
        self.default_listen_addr.clone()
    }

    pub fn contains_addr(&self, other: &HostWithPort) -> bool {
        self.listen_addrs.iter().any(|addr| {
            addr.host == other.host
                && addr.port.unwrap_or_else(|| 5060.into())
                    == other.port.unwrap_or_else(|| 5060.into())
        })
    }
}

fn figure_out_listen_addrs(listen_env_addrs: Option<String>) -> (HostWithPort, Vec<HostWithPort>) {
    match listen_env_addrs {
        Some(listen_env_addrs) => match listen_env_addrs
            .split(',')
            .map(TryInto::try_into)
            .collect::<Result<Vec<HostWithPort>, rsip::Error>>()
        {
            Ok(addrs) if !addrs.is_empty() => (
                addrs.first().cloned().expect("that shouldn't happen"),
                addrs,
            ),
            Ok(_) => {
                log::warn!("Found LISTEN_ADDRS env var but returned nothing");
                let ip_addrs = all_system_ip_addrs();

                (
                    default_ip_addr_from(&ip_addrs).into(),
                    ip_addrs.into_iter().map(Into::into).collect(),
                )
            }
            Err(err) => {
                log::warn!("Failed to parse LISTEN_ADDRS env var: {}", err);
                let ip_addrs = all_system_ip_addrs();

                (
                    default_ip_addr_from(&ip_addrs).into(),
                    ip_addrs.into_iter().map(Into::into).collect(),
                )
            }
        },
        None => {
            log::warn!("missing LISTEN_ADDRS env var, will use default system IPs");
            let ip_addrs = all_system_ip_addrs();

            (
                default_ip_addr_from(&ip_addrs).into(),
                ip_addrs.into_iter().map(Into::into).collect(),
            )
        }
    }
}

fn default_ip_addr_from(ip_addrs: &[IpAddr]) -> IpAddr {
    let default_ip_addr = ip_addrs
        .iter()
        .find(|ip| !ip.is_loopback() && !ip.is_multicast());

    *default_ip_addr.unwrap_or_else(|| ip_addrs.first().expect("foo"))
}

fn all_system_ip_addrs() -> Vec<IpAddr> {
    pnet::datalink::interfaces()
        .into_iter()
        .map(|i| i.ips)
        .flatten()
        .map(|net| net.ip())
        .collect::<Vec<IpAddr>>()
}
