#[macro_use]
pub extern crate diesel;
mod schema;

use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use once_cell::sync::Lazy;
use std::sync::Arc;

mod dialog;
mod error;
mod registration;
mod request;
mod response;
mod transaction;

pub use dialog::{
    Dialog, DialogFlow, DialogWithTransaction, DirtyDialog, DirtyDialogWithTransaction,
};
pub use error::Error;
pub use registration::{DirtyRegistration, Registration, TransportType};
pub use request::{DirtyRequest, Request};
pub use response::{DirtyResponse, Response};
pub use transaction::{DirtyTransaction, Transaction, TransactionState};

type DbConn =
    diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::pg::PgConnection>>;

static DB_POOL: Lazy<Arc<Pool<diesel::r2d2::ConnectionManager<PgConnection>>>> = Lazy::new(|| {
    let manager = ConnectionManager::<PgConnection>::new(
        std::env::var("DATABASE_URL")
            .map_err(|_| String::from("Environment variable Database URL could not be read"))
            .unwrap(),
    );
    Arc::new(
        Pool::builder()
            .max_size(20)
            .build(manager)
            .expect("build pool"),
    )
});

pub fn db_conn() -> Result<DbConn, r2d2::Error> {
    DB_POOL.get()
}
