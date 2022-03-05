use common::{rsip, tokio::time::Instant};

#[derive(Debug)]
pub struct Errored {
    //TODO: Fix me to proper error
    pub error: String,
    pub response: Option<rsip::Response>,
    pub entered_at: Instant,
}
