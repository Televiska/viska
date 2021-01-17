pub mod uac_tests;
pub mod uas_tests;

use crate::common::snitches::{CoreSnitch, TransportSnitch};
use sip_server::{Core, CoreLayer, SipBuilder, SipManager, Transaction};
use std::sync::Arc;

async fn setup() -> Arc<SipManager> {
    SipBuilder::new::<CoreSnitch, Transaction, TransportSnitch>()
        .expect("sip manager failed")
        .manager
}

/*
pub struct TypedSipManager<'a> {
    core: Arc<&'a CoreSnitch>,
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
        let core_match: Option<&CoreSnitch> = sip_manager.core.clone().as_any().clone().downcast_ref::<CoreSnitch>();
        let core: Arc<&CoreSnitch> = match core_match {
            Some(concrete_type) => Arc::new(concrete_type),
            None => {
                panic!("cant't cast value!");
            }
        };

        Self {
            core,
            sip_manager,
        }
    }
}*/
