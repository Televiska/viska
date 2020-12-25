use tokio::time::Instant;

#[derive(Debug)]
pub struct Terminated {
    //final response, if none it means that it timedout
    pub response: Option<rsip::Response>,
    pub entered_at: Instant,
}
