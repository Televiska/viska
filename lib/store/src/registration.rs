use crate::schema::registrations;
use crate::{db_conn, Error};
use common::{
    chrono::{DateTime, Utc},
    ipnetwork::IpNetwork,
};
use diesel::{
    deserialize::FromSql,
    pg::Pg,
    prelude::*,
    serialize::{Output, ToSql},
    sql_types::Text,
};
use std::{
    fmt::{self, Debug},
    io::Write,
};

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
    pub reg_id: i32,
    pub ip_address: IpNetwork,
    pub port: i16,
    pub transport: Transport,
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
    pub reg_id: Option<i32>,
    pub ip_address: Option<IpNetwork>,
    pub port: Option<i16>,
    pub transport: Option<Transport>,
}

pub struct LazyQuery {
    query: registrations::BoxedQuery<'static, diesel::pg::Pg>,
}

impl LazyQuery {
    pub fn new(query: registrations::BoxedQuery<'static, diesel::pg::Pg>) -> Self {
        Self { query }
    }

    pub fn paginate(mut self, page: i64, per_page: i64) -> Self {
        let offset = (page - 1) * per_page;

        self.query = self.query.offset(offset).limit(per_page);
        self
    }

    pub fn username(mut self, username: Option<String>) -> Self {
        if let Some(username) = username {
            self.query = self.query.filter(registrations::username.eq(username));
        }
        self
    }

    pub fn domain(mut self, domain: Option<String>) -> Self {
        if let Some(domain) = domain {
            self.query = self.query.filter(registrations::domain.eq(domain));
        }
        self
    }

    pub fn order_by_created_at(mut self) -> Self {
        self.query = self.query.order(registrations::created_at.desc());
        self
    }

    pub fn load(self) -> Result<Vec<Registration>, Error> {
        Ok(self.query.get_results(&db_conn()?)?)
    }

    pub fn first(self) -> Result<Option<Registration>, Error> {
        Ok(self.query.first(&db_conn()?).optional()?)
    }

    pub fn exists(self) -> Result<bool, Error> {
        use diesel::dsl::{exists, select};

        Ok(select(exists(self.query)).get_result(&db_conn()?)?)
    }
}

impl Registration {
    pub fn query() -> LazyQuery {
        LazyQuery::new(registrations::table.into_boxed())
    }

    pub fn find(id: i64) -> Result<Self, Error> {
        Ok(registrations::table.find(id).first::<Self>(&db_conn()?)?)
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

        let existing_record = Self::query()
            .username(record.username.clone())
            .domain(record.domain.clone())
            .first()?;
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

impl Into<models::Registration> for Registration {
    fn into(self) -> models::Registration {
        models::Registration {
            id: self.id,
            created_at: self.created_at,
            updated_at: self.updated_at,
            username: self.username,
            domain: self.domain,
            contact: self.contact,
            expires: self.expires,
            call_id: self.call_id,
            cseq: self.cseq,
            user_agent: self.user_agent,
            instance: self.instance,
            reg_id: self.reg_id,
            ip_address: self.ip_address,
            port: self.port,
            transport: self.transport.into(),
        }
    }
}

impl From<models::Registration> for DirtyRegistration {
    fn from(model: models::Registration) -> Self {
        Self {
            username: Some(model.username),
            domain: model.domain,
            contact: Some(model.contact),
            expires: Some(model.expires),
            call_id: Some(model.call_id),
            cseq: Some(model.cseq),
            user_agent: Some(model.user_agent),
            instance: model.instance,
            reg_id: Some(model.reg_id),
            ip_address: Some(model.ip_address),
            port: Some(model.port),
            transport: Some(model.transport.into()),
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

impl From<models::UpdateRegistration> for DirtyRegistration {
    fn from(model: models::UpdateRegistration) -> Self {
        Self {
            username: Some(model.username),
            domain: model.domain,
            contact: Some(model.contact),
            expires: model.expires,
            call_id: Some(model.call_id),
            cseq: Some(model.cseq),
            user_agent: Some(model.user_agent),
            instance: model.instance,
            ip_address: Some(model.ip_address),
            port: Some(model.port),
            transport: Some(model.transport.into()),
            reg_id: model.reg_id,
        }
    }
}
