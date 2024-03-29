#[macro_use]
pub extern crate diesel;
pub mod schema;

use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use once_cell::sync::Lazy;
use std::sync::Arc;

mod auth_request;
//mod dialog;
mod error;
mod registration;
mod request;
mod response;
mod transaction;

pub use auth_request::{AuthRequest, DirtyAuthRequest};
//pub use dialog::{
//    Dialog, DialogFlow, DialogWithTransaction, DirtyDialog, DirtyDialogWithTransaction,
//};
pub use error::Error;
pub use registration::{DirtyRegistration, Registration, Transport};
pub use request::{DirtyRequest, Request};
pub use response::{DirtyResponse, Response};
pub use transaction::{DirtyTransaction, Transaction, TransactionState};

//type PgConn = diesel_logger::LoggingConnection<PgConnection>;
type PgConn = PgConnection;
pub type DbConn = diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<PgConn>>;

static DB_POOL: Lazy<Arc<Pool<diesel::r2d2::ConnectionManager<PgConn>>>> = Lazy::new(|| {
    let config = common::Config::default();
    let manager = ConnectionManager::<PgConn>::new(config.database_url);

    Arc::new(
        Pool::builder()
            .max_size(20)
            .build(manager)
            .expect("failed to build database pool"),
    )
});

pub fn db_conn() -> Result<DbConn, r2d2::Error> {
    DB_POOL.get()
}
