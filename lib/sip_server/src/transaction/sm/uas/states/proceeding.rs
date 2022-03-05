use common::tokio::time::Instant;

#[derive(Debug)]
pub struct Proceeding {
    pub entered_at: Instant,
}

impl Default for Proceeding {
    fn default() -> Self {
        Self {
            entered_at: Instant::now(),
        }
    }
}
