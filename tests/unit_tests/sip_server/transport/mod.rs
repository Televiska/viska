pub mod processor;

use crate::common::snitches::UaSnitch;
use sip_server::{SipBuilder, SipManager, Transaction, Transport};
use std::sync::Arc;

async fn setup() -> Arc<SipManager> {
    SipBuilder::new::<UaSnitch, Transaction, Transport>()
        .expect("sip manager failed")
        .manager
}
