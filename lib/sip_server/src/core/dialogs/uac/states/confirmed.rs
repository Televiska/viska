use tokio::time::Instant;

#[derive(Debug)]
pub struct Confirmed {
    pub entered_at: Instant,
}
