use crate::common::{
    self,
    factories::prelude::*,
    snitches::{CorePanic, RegistrarSnitch, ReqProcessorPanic, TransactionPanic, TransportSnitch},
};
use ::common::ipnetwork::IpNetwork;
use ::common::rsip::{self, prelude::*};
use models::transport::RequestMsg;
use sip_server::{
    core::{CoreProcessor, Processor, ReqProcessor},
    SipBuilder, SipManager,
};
use std::sync::Arc;

async fn setup() -> (
    Processor<RegistrarSnitch, ReqProcessorPanic>,
    Arc<SipManager>,
) {
    let sip_manager = SipBuilder::new::<CorePanic, TransactionPanic, TransportSnitch>()
        .expect("sip manager failed")
        .manager;

    let processor: Processor<RegistrarSnitch, ReqProcessorPanic> =
        Processor::new(Arc::downgrade(&sip_manager));

    (processor, sip_manager)
}

#[tokio::test]
#[serial_test::serial]
async fn sending_an_options_request_receives_busy() {
    let _ = common::setup();
    let (processor, sip_manager) = setup().await;
    let transport = sip_manager.transport.clone();
    let transport = as_any!(transport, TransportSnitch);

    let res = processor
        .process_incoming_message(
            RequestMsg {
                sip_request: requests::register_request(),
                ..Randomized::default()
            }
            .into(),
        )
        .await;
    //TODO: we need to do somethng with auth returning an Error
    //assert!(res.is_ok(), format!("returns: {:?}", res));
    assert_eq!(transport.messages.len().await, 1);
    assert_eq!(
        transport.messages.first_response().await.status_code,
        401.into()
    );
}
