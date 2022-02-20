pub mod uac_tests;
pub mod uas_tests;

use crate::common::snitches::{UaSnitch, TransportSnitch};
use sip_server::{tu::elements::UserAgent, SipBuilder, SipManager, Transaction, TuLayer};
use std::sync::Arc;

async fn setup() -> Arc<SipManager> {
    SipBuilder::new::<UaSnitch, Transaction, TransportSnitch>()
        .expect("sip manager failed")
        .manager
}

/*
pub struct TypedSipManager<'a> {
    ua: Arc<&'a UaSnitch>,
    /*
    transaction: &'b Transaction,
    transport: &'c TransportSnitch,
    */
    sip_manager: Arc<SipManager>,
}

impl From<Arc<SipManager>> for TypedSipManager<'_> {
    fn from(sip_manager: Arc<SipManager>) -> Self {
        /*
        let transport = sip_manager.transport.clone();
        let transport = as_any!(transport, TransportSnitch);
        let transaction = sip_manager.transaction.clone();
        let transaction = as_any!(transaction, Transaction);
        */
        let ua_match: Option<&UaSnitch> = sip_manager.tu.clone().as_any().clone().downcast_ref::<UaSnitch>();
        let ua: Arc<&UaSnitch> = match ua_match {
            Some(concrete_type) => Arc::new(concrete_type),
            None => {
                panic!("cant't cast value!");
            }
        };

        Self {
            ua,
            sip_manager,
        }
    }
}*/
