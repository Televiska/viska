use common::tokio::time::Instant;

#[derive(Debug)]
pub struct Terminated {
    pub entered_at: Instant,
}
