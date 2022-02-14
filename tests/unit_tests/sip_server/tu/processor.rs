use crate::common::{
    self,
    factories::prelude::*,
    snitches::{UaPanic, RegistrarSnitch, ReqProcessorPanic, TransactionPanic, TransportSnitch, DialogsEmptySnitch},
};
use ::common::ipnetwork::IpNetwork;
use ::common::rsip::{self, prelude::*};
use models::transport::RequestMsg;
use sip_server::{tu::impls::UaProcessor, ReqProcessor, SipBuilder, SipManager, TuProcessor};
use std::sync::Arc;

async fn setup() -> (
    UaProcessor<RegistrarSnitch, ReqProcessorPanic, DialogsEmptySnitch>,
    Arc<SipManager>,
) {
    let sip_manager = SipBuilder::new::<UaPanic, TransactionPanic, TransportSnitch>()
        .expect("sip manager failed")
        .manager;

    let processor: UaProcessor<RegistrarSnitch, ReqProcessorPanic, DialogsEmptySnitch> =
        UaProcessor::new(Arc::downgrade(&sip_manager));

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
