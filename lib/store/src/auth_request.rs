use crate::schema::auth_requests;
use crate::{db_conn, Error};
use common::{
    chrono::{DateTime, Utc},
    uuid::Uuid,
};
use diesel::prelude::*;

#[derive(Queryable, AsChangeset, Insertable, Debug, Clone)]
#[table_name = "auth_requests"]
pub struct AuthRequest {
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub nonce: String,
    pub consumed_at: Option<DateTime<Utc>>,
}

#[derive(AsChangeset, Insertable, Debug)]
#[table_name = "auth_requests"]
pub struct DirtyAuthRequest {
    pub nonce: Option<String>,
    pub consumed_at: Option<DateTime<Utc>>,
}

impl Default for DirtyAuthRequest {
    fn default() -> Self {
        Self {
            nonce: Some(Uuid::new_v4().to_string()),
            consumed_at: None,
        }
    }
}

pub struct LazyQuery {
    query: auth_requests::BoxedQuery<'static, diesel::pg::Pg>,
}

impl LazyQuery {
    pub fn new(query: auth_requests::BoxedQuery<'static, diesel::pg::Pg>) -> Self {
        Self { query }
    }

    pub fn paginate(mut self, page: i64, per_page: i64) -> Self {
        let offset = (page - 1) * per_page;

        self.query = self.query.offset(offset).limit(per_page);
        self
    }

    pub fn nonce(mut self, nonce: Option<String>) -> Self {
        if let Some(nonce) = nonce {
            self.query = self.query.filter(auth_requests::nonce.eq(nonce));
        }
        self
    }

    pub fn order_by_created_at(mut self) -> Self {
        self.query = self.query.order(auth_requests::created_at.desc());
        self
    }

    pub fn load(self) -> Result<Vec<AuthRequest>, Error> {
        Ok(self.query.get_results(&db_conn()?)?)
    }

    pub fn first(self) -> Result<Option<AuthRequest>, Error> {
        Ok(self.query.first(&db_conn()?).optional()?)
    }

    pub fn exists(self) -> Result<bool, Error> {
        use diesel::dsl::{exists, select};

        Ok(select(exists(self.query)).get_result(&db_conn()?)?)
    }
}

impl AuthRequest {
    pub fn query() -> LazyQuery {
        LazyQuery::new(auth_requests::table.into_boxed())
    }

    pub fn find(id: i64) -> Result<Self, Error> {
        Ok(auth_requests::table.find(id).first::<Self>(&db_conn()?)?)
    }

    pub fn create(record: impl Into<DirtyAuthRequest>) -> Result<Self, Error> {
        use diesel::insert_into;

        Ok(insert_into(auth_requests::table)
            .values(record.into())
            .get_result(&db_conn()?)?)
    }

    pub fn update(record: impl Into<DirtyAuthRequest>, id: i64) -> Result<Self, Error> {
        Ok(
            diesel::update(auth_requests::table.filter(auth_requests::id.eq(id)))
                .set(&record.into())
                .get_result(&db_conn()?)?,
        )
    }

    pub fn consumed(nonce: String) -> Result<Self, Error> {
        Ok(
            diesel::update(auth_requests::table.filter(auth_requests::nonce.eq(nonce)))
                .set(auth_requests::consumed_at.eq(Utc::now()))
                .get_result(&db_conn()?)?,
        )
    }

    pub fn delete(id: i64) -> Result<Self, Error> {
        Ok(
            diesel::delete(auth_requests::table.filter(auth_requests::id.eq(id)))
                .get_result(&db_conn()?)?,
        )
    }
}

#[allow(clippy::from_over_into)]
impl Into<models::AuthRequest> for AuthRequest {
    fn into(self) -> models::AuthRequest {
        models::AuthRequest {
            id: self.id,
            created_at: self.created_at,
            updated_at: self.updated_at,
            nonce: self.nonce,
            consumed_at: None,
        }
    }
}
