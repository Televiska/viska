use common::{rsip, tokio::time::Instant};

#[derive(Debug)]
pub enum Terminated {
    Expected {
        //response: rsip::Response,
        entered_at: Instant,
    },
    TimedOut {
        response: Option<rsip::Response>,
        entered_at: Instant,
    },
    Errored {
        error: String,
        sip_message: Option<rsip::SipMessage>,
        entered_at: Instant,
    },
}
