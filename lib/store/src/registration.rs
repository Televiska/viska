use crate::schema::registrations;
use crate::{db_conn, Error};
use common::{
    chrono::{DateTime, Duration, Utc},
    ipnetwork::IpNetwork,
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
    convert::{TryFrom, TryInto},
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

#[derive(FromSqlRow, AsExpression, Copy, Clone, PartialEq, Debug)]
#[sql_type = "Text"]
pub enum Transport {
    Tcp,
    Udp,
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
            s => Err(format!("invalid Transport `{}`", s)),
        }
    }
}

impl Into<rsip::common::Transport> for Transport {
    fn into(self) -> rsip::common::Transport {
        match self {
            Transport::Tcp => rsip::common::Transport::Tcp,
            Transport::Udp => rsip::common::Transport::Udp,
        }
    }
}

impl From<rsip::common::Transport> for Transport {
    fn from(model: rsip::common::Transport) -> Transport {
        match model {
            rsip::common::Transport::Tcp => Transport::Tcp,
            rsip::common::Transport::Udp => Transport::Udp,
        }
    }
}

impl TryFrom<RequestMsg> for DirtyRegistration {
    type Error = crate::Error;

    fn try_from(msg: RequestMsg) -> Result<Self, Self::Error> {
        use rsip::{
            common::Method,
            message::{ExpiresExt, HeadersExt},
        };

        let request = msg.sip_request;

        if request.method != Method::Register {
            return Err(Self::Error::custom(format!(
                "cannot create registration from {} method",
                request.method
            )));
        }
        let expires = match (request.contact_header_expires()?, request.expires_header()) {
            (Some(expire), _) => expire,
            (None, Ok(rsip::headers::Expires(expire))) => *expire,
            _ => 3600,
        } as i64;

        let contact_header = request.contact_header()?;

        Ok(Self {
            username: Some(
                request
                    .from_header()?
                    .0
                    .uri
                    .username()
                    .ok_or("missing username in from header")?,
            ),
            domain: Some(request.from_header()?.clone().0.uri.domain()),
            contact: Some(
                Into::<rsip::Header>::into(contact_header.clone())
                    .to_string()
                    .splitn(2, ':')
                    .last()
                    .expect("contact header value part")
                    .to_owned(),
            ),
            expires: Some(Utc::now() + Duration::seconds(expires)),
            call_id: Some(request.call_id_header()?.clone().0),
            cseq: Some(request.cseq_header()?.seq as i32),
            user_agent: Some(request.user_agent_header()?.clone().0),
            instance: request.contact_header()?.sip_instance(),
            ip_address: Some(IpAddr::V4(Ipv4Addr::new(192, 168, 0, 3)).into()),
            port: Some(request.from_header()?.clone().0.uri.port() as i16),
            contact_uri: Some(contact_header.0.uri.to_string()),
            transport: Some(Transport::Udp),
        })
    }
}

impl TryInto<rsip::headers::Contact> for Registration {
    type Error = rsip::Error;

    fn try_into(self) -> Result<rsip::headers::Contact, Self::Error> {
        Ok(self.contact.try_into()?)
    }
}
