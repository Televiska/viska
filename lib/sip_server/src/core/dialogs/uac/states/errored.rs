use tokio::time::Instant;

#[derive(Debug)]
pub struct Errored {
    pub entered_at: Instant,
}

