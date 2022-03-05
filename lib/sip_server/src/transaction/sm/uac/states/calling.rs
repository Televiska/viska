use common::tokio::time::Instant;
use std::time::Duration;

use super::super::{TIMER_B, TIMER_T1};

#[derive(Debug, Clone, Copy)]
pub struct Calling {
    pub entered_at: Instant,
    pub retransmissions_count: u8,
    pub last_retransmission_at: Instant,
}

impl Calling {
    pub fn next_retrasmission(&self) -> Duration {
        use std::iter;

        iter::repeat(Duration::from_millis(TIMER_T1))
            .take(2_i32.pow(self.retransmissions_count.into()) as usize)
            .fold(Duration::from_secs(0), |acc, x| acc + x)
    }

    pub fn has_timedout(&self) -> bool {
        self.entered_at.elapsed() >= Duration::from_millis(TIMER_B)
    }

    pub fn should_retransmit(&self) -> bool {
        self.last_retransmission_at.elapsed() > self.next_retrasmission()
    }

    pub fn retransmit(self) -> Self {
        Self {
            retransmissions_count: self.retransmissions_count + 1,
            last_retransmission_at: Instant::now(),
            ..self
        }
    }
}

impl Default for Calling {
    fn default() -> Self {
        Self {
            entered_at: Instant::now(),
            retransmissions_count: 0,
            last_retransmission_at: Instant::now(),
        }
    }
}
