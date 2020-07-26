use crate::schema::responses;
use crate::{db_conn, Error};
use common::chrono::{DateTime, Utc};
use diesel::prelude::*;

#[derive(Queryable, AsChangeset, Insertable, Debug, Clone)]
#[table_name = "responses"]
pub struct Response {
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub code: i16,
    pub headers: String,
    pub body: Option<String>,
    pub raw_message: Option<String>,
}

#[derive(AsChangeset, Insertable, Debug, Default)]
#[table_name = "responses"]
pub struct DirtyResponse {
    pub code: Option<i16>,
    pub headers: Option<String>,
    pub body: Option<String>,
    pub raw_message: Option<String>,
}

pub struct LazyQuery {
    query: responses::BoxedQuery<'static, diesel::pg::Pg>,
}

impl LazyQuery {
    pub fn new(query: responses::BoxedQuery<'static, diesel::pg::Pg>) -> Self {
        Self { query }
    }

    pub fn paginate(mut self, page: i64, per_page: i64) -> Self {
        let offset = (page - 1) * per_page;

        self.query = self.query.offset(offset).limit(per_page);
        self
    }

    pub fn order_by_created_at(mut self) -> Self {
        self.query = self.query.order(responses::created_at.desc());
        self
    }

    pub async fn load(self) -> Result<Vec<Response>, Error> {
        Ok(self.query.get_results(&db_conn()?)?)
    }

    pub async fn first(self) -> Result<Response, Error> {
        Ok(self.query.first(&db_conn()?)?)
    }
}

impl Response {
    pub fn query() -> LazyQuery {
        LazyQuery::new(responses::table.into_boxed())
    }

    pub async fn find(id: i64) -> Result<Self, Error> {
        Ok(responses::table.find(id).first::<Self>(&db_conn()?)?)
    }

    pub async fn create(record: DirtyResponse) -> Result<Self, Error> {
        use diesel::insert_into;

        Ok(insert_into(responses::table)
            .values(record)
            .get_result(&db_conn()?)?)
    }

    pub async fn update(record: DirtyResponse, id: i64) -> Result<Self, Error> {
        Ok(
            diesel::update(responses::table.filter(responses::id.eq(id)))
                .set(&record)
                .get_result(&db_conn()?)?,
        )
    }

    pub async fn delete(id: i64) -> Result<Self, Error> {
        Ok(diesel::delete(responses::table.filter(responses::id.eq(id))).get_result(&db_conn()?)?)
    }
}
