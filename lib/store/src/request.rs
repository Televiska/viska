use crate::schema::requests;
use crate::{db_conn, Error};
use common::chrono::{DateTime, Utc};
use diesel::prelude::*;

#[derive(Queryable, AsChangeset, Insertable, Debug, Clone)]
#[table_name = "requests"]
pub struct Request {
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub method: String,
    pub uri: String,
    pub headers: String,
    pub body: Option<String>,
    pub raw_message: Option<String>,
}

#[derive(AsChangeset, Insertable, Debug, Default)]
#[table_name = "requests"]
pub struct DirtyRequest {
    pub method: Option<String>,
    pub uri: Option<String>,
    pub headers: Option<String>,
    pub body: Option<String>,
    pub raw_message: Option<String>,
}

pub struct LazyQuery {
    query: requests::BoxedQuery<'static, diesel::pg::Pg>,
}

impl LazyQuery {
    pub fn new(query: requests::BoxedQuery<'static, diesel::pg::Pg>) -> Self {
        Self { query }
    }

    pub fn paginate(mut self, page: i64, per_page: i64) -> Self {
        let offset = (page - 1) * per_page;

        self.query = self.query.offset(offset).limit(per_page);
        self
    }

    pub fn order_by_created_at(mut self) -> Self {
        self.query = self.query.order(requests::created_at.desc());
        self
    }

    pub async fn load(self) -> Result<Vec<Request>, Error> {
        Ok(self.query.get_results(&db_conn()?)?)
    }

    pub async fn first(self) -> Result<Request, Error> {
        Ok(self.query.first(&db_conn()?)?)
    }
}

impl Request {
    pub fn query() -> LazyQuery {
        LazyQuery::new(requests::table.into_boxed())
    }

    pub async fn find(id: i64) -> Result<Self, Error> {
        Ok(requests::table.find(id).first::<Self>(&db_conn()?)?)
    }

    pub async fn create(record: DirtyRequest) -> Result<Self, Error> {
        use diesel::insert_into;

        Ok(insert_into(requests::table)
            .values(record)
            .get_result(&db_conn()?)?)
    }

    pub async fn update(record: DirtyRequest, id: i64) -> Result<Self, Error> {
        Ok(diesel::update(requests::table.filter(requests::id.eq(id)))
            .set(&record)
            .get_result(&db_conn()?)?)
    }

    pub async fn delete(id: i64) -> Result<Self, Error> {
        Ok(diesel::delete(requests::table.filter(requests::id.eq(id))).get_result(&db_conn()?)?)
    }
}
