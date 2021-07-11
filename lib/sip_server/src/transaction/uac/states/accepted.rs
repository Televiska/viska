use common::rsip::prelude::*;
use std::time::Duration;
use tokio::time::Instant;

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
