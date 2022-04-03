use common::{rsip, tokio::time::Instant};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Proceeding {
    pub response: rsip::Response,
    pub entered_at: Instant,
}

impl Proceeding {
    pub fn with_response(&mut self, response: rsip::Response) {
        self.response = response;
    }
}
