use tokio::time::Instant;

#[derive(Debug)]
pub struct Proceeding {
    pub response: rsip::Response,
    pub entered_at: Instant,
}

impl Proceeding {
    pub fn with_response(&mut self, response: rsip::Response) {
        self.response = response;
    }
}
