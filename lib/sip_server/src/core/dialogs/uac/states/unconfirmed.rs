use tokio::time::Instant;

#[derive(Debug)]
pub struct Unconfirmed {
    pub entered_at: Instant,
}

impl Default for Unconfirmed {
    fn default() -> Self {
        Self {
            entered_at: Instant::now(),
        }
    }
}
