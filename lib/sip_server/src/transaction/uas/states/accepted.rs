use std::time::Duration;
use tokio::time::Instant;

use super::super::TIMER_L;

#[derive(Debug)]
pub struct Accepted {
    pub entered_at: Instant,
}

impl Accepted {
    pub fn should_terminate(&self) -> bool {
        self.entered_at.elapsed() > Duration::from_millis(TIMER_L)
    }
}

impl Default for Accepted {
    fn default() -> Self {
        Self {
            entered_at: Instant::now(),
        }
    }
}
