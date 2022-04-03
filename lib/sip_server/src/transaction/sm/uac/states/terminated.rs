use common::{rsip, tokio::time::Instant};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Terminated {
    pub timed_out: bool,
    pub response: Option<rsip::Response>,
    pub entered_at: Instant,
}
