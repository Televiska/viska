use common::{
    chrono::{DateTime, Utc},
};
//use sip_helpers::auth::WwwAuthenticationHeader;

#[derive(Debug, Clone)]
pub struct AuthRequest {
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub nonce: String,
    pub consumed_at: Option<DateTime<Utc>>
}
