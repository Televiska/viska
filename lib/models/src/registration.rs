use common::{
    chrono::{DateTime, Utc},
    ipnetwork::IpNetwork,
};

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
    pub transport: TransportType,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum TransportType {
    Tcp,
    Udp,
}

impl From<store::Registration> for Registration {
    fn from(record: store::Registration) -> Self {
        Self {
            id: record.id,
            created_at: record.created_at,
            updated_at: record.updated_at,
            username: record.username,
            domain: record.domain,
            contact: record.contact,
            expires: record.expires,
            call_id: record.call_id,
            cseq: record.cseq,
            user_agent: record.user_agent,
            instance: record.instance,
            reg_id: record.reg_id,
            ip_address: record.ip_address,
            port: record.port,
            transport: record.transport.into(),
        }
    }
}

impl Into<store::DirtyRegistration> for Registration {
    fn into(self) -> store::DirtyRegistration {
        store::DirtyRegistration {
            username: Some(self.username),
            domain: self.domain,
            contact: Some(self.contact),
            expires: Some(self.expires),
            call_id: Some(self.call_id),
            cseq: Some(self.cseq),
            user_agent: Some(self.user_agent),
            instance: self.instance,
            reg_id: Some(self.reg_id),
            ip_address: Some(self.ip_address),
            port: Some(self.port),
            transport: Some(self.transport.into()),
        }
    }
}

impl From<store::TransportType> for TransportType {
    fn from(transport_type: store::TransportType) -> Self {
        match transport_type {
            store::TransportType::Tcp => Self::Tcp,
            store::TransportType::Udp => Self::Udp,
        }
    }
}

impl Into<store::TransportType> for TransportType {
    fn into(self) -> store::TransportType {
        match self {
            Self::Tcp => store::TransportType::Tcp,
            Self::Udp => store::TransportType::Udp,
        }
    }
}
