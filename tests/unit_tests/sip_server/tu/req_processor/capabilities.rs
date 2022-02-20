use crate::common::{
    self,
    factories::prelude::*,
    snitches::{UaSnitch, TransportSnitch},
};
use ::common::ipnetwork::IpNetwork;
use ::common::rsip::{self, prelude::*};
use models::transport::RequestMsg;
use sip_server::{
    tu::elements::{Capabilities, UserAgent},
    ReqProcessor, SipBuilder, SipManager, Transaction, TuLayer,
};
use std::sync::Arc;

async fn setup() -> (Capabilities, Arc<SipManager>) {
    let sip_manager = SipBuilder::new::<UaSnitch, Transaction, TransportSnitch>()
        .expect("sip manager failed")
        .manager;

    let capabilities = Capabilities::new(Arc::downgrade(&sip_manager));

    (capabilities, sip_manager)
}

#[tokio::test]
#[serial_test::serial]
async fn sending_an_options_request_receives_busy() {
    let _ = common::setup();
    let (capabilities, sip_manager) = setup().await;
    let transport = sip_manager.transport.clone();
    let transport = as_any!(transport, TransportSnitch);

    let res = capabilities
        .process_incoming_request(RequestMsg {
            sip_request: requests::options_request(),
            ..Randomized::default()
        })
        .await;
    assert!(res.is_ok(), "returns: {:?}", res);
    assert_eq!(transport.messages.len().await, 1);
    assert_eq!(
        transport.messages.first_response().await.status_code,
        486.into()
    );
}
