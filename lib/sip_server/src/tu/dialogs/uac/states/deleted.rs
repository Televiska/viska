use common::tokio::time::Instant;

#[derive(Debug)]
pub struct Deleted {
    pub entered_at: Instant,
}
