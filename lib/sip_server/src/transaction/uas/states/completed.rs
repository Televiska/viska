use std::time::Duration;
use tokio::time::Instant;

use super::super::{TIMER_G, TIMER_H, TIMER_T2};

#[derive(Debug, Clone, Copy)]
pub struct Completed {
    pub entered_at: Instant,
    pub retransmissions_count: u8,
    pub last_retransmission_at: Instant,
}

impl Completed {
    pub fn next_retrasmission(&self) -> Duration {
        use std::iter;

        std::cmp::min(
            iter::repeat(Duration::from_millis(TIMER_G))
                .take(2_i32.pow(self.retransmissions_count.into()) as usize)
                .fold(Duration::from_secs(0), |acc, x| acc + x),
            Duration::from_millis(TIMER_T2),
        )
    }

    pub fn has_timedout(&self) -> bool {
        self.entered_at.elapsed() >= Duration::from_millis(TIMER_H)
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

impl Default for Completed {
    fn default() -> Self {
        Self {
            entered_at: Instant::now(),
            retransmissions_count: 0,
            last_retransmission_at: Instant::now(),
        }
    }
}
