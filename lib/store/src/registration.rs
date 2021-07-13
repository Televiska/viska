use crate::schema::registrations;
use crate::{db_conn, Error};
use common::{
    chrono::{DateTime, Duration, Utc},
    ipnetwork::IpNetwork,
    rsip::prelude::*,
};
use diesel::{
    deserialize::FromSql,
    pg::Pg,
    prelude::*,
    serialize::{Output, ToSql},
    sql_types::Text,
};
use models::transport::RequestMsg;
use std::{
    convert::TryFrom,
    fmt::{self, Debug},
    io::Write,
    net::{IpAddr, Ipv4Addr},
};

#[derive(Debug, Default)]
pub struct SearchFilter {
    pub id: Option<i64>,
    pub username: Option<String>,
    pub domain: Option<String>,
    pub offset: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Queryable, AsChangeset, Insertable, Debug, Clone)]
#[table_name = "registrations"]
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
    pub ip_address: IpNetwork,
    pub port: i16,
    pub transport: Transport,
    pub contact_uri: String,
}

#[derive(AsChangeset, Insertable, Debug, Default)]
#[table_name = "registrations"]
pub struct DirtyRegistration {
    pub username: Option<String>,
    pub domain: Option<String>,
    pub contact: Option<String>,
    pub expires: Option<DateTime<Utc>>,
    pub call_id: Option<String>,
    pub cseq: Option<i32>,
    pub user_agent: Option<String>,
    pub instance: Option<String>,
    pub ip_address: Option<IpNetwork>,
    pub port: Option<i16>,
    pub transport: Option<Transport>,
    pub contact_uri: Option<String>,
}

impl Registration {
    fn query_boxed(filter: SearchFilter) -> registrations::BoxedQuery<'static, diesel::pg::Pg> {
        let mut query = registrations::table.into_boxed();

        if let Some(id) = filter.id {
            query = query.filter(registrations::id.eq(id));
        }

        if let Some(username) = filter.username {
            query = query.filter(registrations::username.eq(username));
        }

        if let Some(domain) = filter.domain {
            query = query.filter(registrations::domain.eq(domain));
        }

        if let Some(offset) = filter.offset {
            query = query.offset(offset)
        }

        if let Some(per_page) = filter.per_page {
            query = query.limit(per_page)
        }

        query
    }

    pub fn search(filter: SearchFilter) -> Result<Vec<Registration>, Error> {
        Ok(Self::query_boxed(filter).load::<Registration>(&db_conn()?)?)
    }

    pub fn count(filter: SearchFilter) -> Result<i64, Error> {
        Ok(Self::query_boxed(filter).count().get_result(&db_conn()?)?)
    }

    pub fn find_by(filter: SearchFilter) -> Result<Option<Registration>, Error> {
        Ok(Self::query_boxed(filter)
            .get_result::<Registration>(&db_conn()?)
            .optional()?)
    }

    pub fn find(id: i64) -> Result<Registration, Error> {
        Ok(registrations::table
            .filter(registrations::id.eq(id))
            .get_result::<Registration>(&db_conn()?)?)
    }

    pub fn create(record: impl Into<DirtyRegistration>) -> Result<Self, Error> {
        use diesel::insert_into;

        Ok(insert_into(registrations::table)
            .values(record.into())
            .get_result(&db_conn()?)?)
    }

    //TODO: fix me by adding proper indexes and using proper ON CONFLICT clauses
    pub fn upsert(record: impl Into<DirtyRegistration>) -> Result<Self, Error> {
        let record = record.into();

        let existing_record = Self::find_by(SearchFilter {
            username: record.username.clone(),
            domain: record.domain.clone(),
            ..Default::default()
        })?;
        match existing_record {
            Some(existing_record) => Ok(Self::update(record, existing_record.id)?),
            None => Ok(Self::create(record)?),
        }
    }

    pub fn update(record: impl Into<DirtyRegistration>, id: i64) -> Result<Self, Error> {
        Ok(
            diesel::update(registrations::table.filter(registrations::id.eq(id)))
                .set(&record.into())
                .get_result(&db_conn()?)?,
        )
    }

    pub fn delete(id: i64) -> Result<Self, Error> {
        Ok(
            diesel::delete(registrations::table.filter(registrations::id.eq(id)))
                .get_result(&db_conn()?)?,
        )
    }

    pub fn delete_by_uri(uri: String) -> Result<Self, Error> {
        Ok(
            diesel::delete(registrations::table.filter(registrations::contact_uri.eq(uri)))
                .get_result(&db_conn()?)?,
        )
    }
}

#[derive(FromSqlRow, AsExpression, Clone, PartialEq, Debug)]
#[sql_type = "Text"]
pub enum Transport {
    Tcp,
    Udp,
    Tls,
    Sctp,
}
impl fmt::Display for Transport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(self, f)
    }
}
impl ToSql<Text, Pg> for Transport {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> diesel::serialize::Result {
        <&str as ToSql<Text, Pg>>::to_sql(&self.to_string().to_lowercase().as_str(), out)
    }
}
impl FromSql<Text, Pg> for Transport {
    fn from_sql(bytes: Option<diesel::pg::PgValue>) -> diesel::deserialize::Result<Self> {
        use std::str::FromStr;

        Ok(Transport::from_str(
            <String as FromSql<Text, Pg>>::from_sql(bytes)?.as_ref(),
        )?)
    }
}

impl std::str::FromStr for Transport {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            s if s.eq_ignore_ascii_case("tcp") => Ok(Transport::Tcp),
            s if s.eq_ignore_ascii_case("udp") => Ok(Transport::Udp),
            s if s.eq_ignore_ascii_case("tls") => Ok(Transport::Tls),
            s if s.eq_ignore_ascii_case("sctp") => Ok(Transport::Sctp),
            s => Err(format!("failed to parse transport {}", s)),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<rsip::common::Transport> for Transport {
    fn into(self) -> rsip::common::Transport {
        match self {
            Transport::Tcp => rsip::common::Transport::Tcp,
            Transport::Udp => rsip::common::Transport::Udp,
            Transport::Tls => rsip::common::Transport::Tls,
            Transport::Sctp => rsip::common::Transport::Sctp,
        }
    }
}

impl From<rsip::common::Transport> for Transport {
    fn from(model: rsip::common::Transport) -> Transport {
        match model {
            rsip::common::Transport::Tcp => Transport::Tcp,
            rsip::common::Transport::Udp => Transport::Udp,
            rsip::common::Transport::Tls => Transport::Tls,
            rsip::common::Transport::Sctp => Transport::Sctp,
        }
    }
}

impl TryFrom<RequestMsg> for DirtyRegistration {
    type Error = crate::Error;

    fn try_from(msg: RequestMsg) -> Result<Self, Self::Error> {
        let request = msg.sip_request;

        if request.method != rsip::common::Method::Register {
            return Err(Self::Error::custom(format!(
                "cannot create registration from {} method",
                request.method
            )));
        }
        let expires = match (
            request
                .contact_header()?
                .typed()?
                .expires()
                .map(|s| s.seconds())
                .transpose()?,
            request
                .expires_header()
                .map(|h| h.typed())
                .transpose()?
                .map(|h| *h.value()),
        ) {
            (Some(expire), _) => expire,
            (None, Some(expire)) => expire,
            _ => 3600,
        } as i64;

        let contact_header = request.contact_header()?;
        let typed_contact_header = contact_header.typed()?;

        Ok(Self {
            username: Some(
                request
                    .from_header()?
                    .typed()?
                    .uri
                    .username()
                    .ok_or("missing username in from header")?
                    .into(),
            ),
            domain: Some(request.from_header()?.typed()?.uri.host().to_string()),
            contact: Some(contact_header.value().into()),
            expires: Some(Utc::now() + Duration::seconds(expires)),
            call_id: Some(request.call_id_header()?.clone().into()),
            cseq: Some(request.cseq_header()?.typed()?.seq as i32),
            user_agent: Some(request.user_agent_header()?.clone().into()),
            instance: Some("something".into()),
            ip_address: Some(IpAddr::V4(Ipv4Addr::new(192, 168, 0, 3)).into()),
            port: Some(
                request
                    .from_header()?
                    .typed()?
                    .uri
                    .port()
                    .map(|s| *s.value() as i16)
                    .unwrap_or(5060),
            ),
            contact_uri: Some(typed_contact_header.uri.to_string()),
            transport: Some(Transport::Udp),
        })
    }
}

#[allow(clippy::from_over_into)]
impl Into<rsip::headers::Contact> for Registration {
    fn into(self) -> rsip::headers::Contact {
        self.contact.into()
    }
}
