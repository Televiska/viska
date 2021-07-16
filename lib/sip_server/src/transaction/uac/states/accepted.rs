use common::{rsip, tokio::time::Instant};
use std::time::Duration;

use super::super::TIMER_M;

#[derive(Debug, Clone)]
pub struct Accepted {
    pub response: rsip::Response,
    pub entered_at: Instant,
}

impl Accepted {
    pub fn should_terminate(&self) -> bool {
        self.entered_at.elapsed() > Duration::from_millis(TIMER_M)
    }
}
