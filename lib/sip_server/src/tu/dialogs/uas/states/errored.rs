use common::{rsip, tokio::time::Instant};

#[derive(Debug)]
pub struct Errored {
    //TODO: Fix me to proper error type
    pub error: String,
    pub sip_message: Option<rsip::SipMessage>,
    pub entered_at: Instant,
}
