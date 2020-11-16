use tokio::time::Instant;

#[derive(Debug)]
pub struct Terminated {
    pub timedout: bool,
    pub entered_at: Instant,
}
