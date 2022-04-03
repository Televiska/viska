use common::{rsip, tokio::time::Instant};

#[derive(Debug)]
pub enum Terminated {
    Expected {
        entered_at: Instant,
    },
    TimedOut {
        //response: Option<rsip::Response>,
        entered_at: Instant,
    },
    Errored {
        error: String,
        response: Option<rsip::Response>,
        entered_at: Instant,
    },
}
