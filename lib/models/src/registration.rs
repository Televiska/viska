use common::{
    chrono::{DateTime, Duration, Utc},
    ipnetwork::{IpNetwork, Ipv4Network},
};
use rsip::common::Transport;
use std::{convert::TryFrom, net::Ipv4Addr};

#[derive(Debug, Clone)]
pub struct Registration {
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub username: String,
    pub domain: Option<String>,
    pub contact: String,
    pub expires: DateTime<Utc>,
    pub call_id: String,
    pub cseq: i32,
    pub user_agent: String,
    pub instance: Option<String>,
    pub reg_id: i32,
    pub ip_address: IpNetwork,
    pub port: i16,
    pub transport: Transport,
}

pub struct UpdateRegistration {
    pub username: String,
    pub domain: Option<String>,
    pub contact: String,
    pub expires: Option<DateTime<Utc>>,
    pub call_id: String,
    pub cseq: i32,
    pub user_agent: String,
    pub instance: Option<String>,
    pub reg_id: Option<i32>,
    pub ip_address: IpNetwork,
    pub port: i16,
    pub transport: Transport,
}

//TODO: figure out params here
impl TryFrom<rsip::Request> for UpdateRegistration {
    type Error = crate::Error;

    fn try_from(request: rsip::Request) -> Result<Self, Self::Error> {
        use rsip::message::{ExpiresExt, HeadersExt};
        Ok(Self {
            username: request
                .from_header()?
                .0
                .uri
                .username()
                .ok_or("missing username")?,
            domain: Some(request.from_header()?.clone().0.uri.domain()),
            contact: Into::<rsip::headers::Header>::into(request.contact_header()?.clone())
                .to_string(),
            expires: Some(
                Utc::now()
                    + Duration::seconds(
                        request
                            .contact_header_expires()?
                            .unwrap_or(request.expires_header()?.0) as i64,
                    ),
            ),
            call_id: request.call_id_header()?.clone().0,
            cseq: request.cseq_header()?.seq as i32,
            user_agent: request.user_agent_header()?.clone().0,
            instance: request.contact_header()?.sip_instance(),
            ip_address: IpNetwork::V4(Ipv4Network::new(Ipv4Addr::new(192, 168, 0, 3), 32)?),
            port: request.from_header()?.clone().0.uri.port() as i16,
            transport: Transport::Udp,
            reg_id: None,
        })
    }
}
