use tokio::time::Instant;

#[derive(Debug)]
pub struct Early {
    pub entered_at: Instant,
}

