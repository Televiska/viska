use common::{rsip, tokio::time::Instant};
use std::time::Duration;

use super::super::TIMER_I;

#[derive(Debug)]
pub struct Confirmed {
    pub request: rsip::Request,
    pub entered_at: Instant,
}

impl Confirmed {
    pub fn should_terminate(&self) -> bool {
        self.entered_at.elapsed() > Duration::from_millis(TIMER_I)
    }
}
