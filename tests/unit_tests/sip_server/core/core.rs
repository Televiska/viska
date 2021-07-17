use crate::common::{
    self,
    factories::prelude::*,
    snitches::{
        CorePanic, DialogsSnitch, RegistrarSnitch, ReqProcessorPanic, TransactionPanic,
        TransportSnitch,
    },
};
use ::common::ipnetwork::IpNetwork;
use ::common::rsip::{self, prelude::*};
use models::transport::RequestMsg;
use sip_server::{
    core::{Core, CoreLayer, ReqProcessor},
    SipBuilder, SipManager,
};
use std::sync::Arc;

async fn setup() -> (
    Core<RegistrarSnitch, ReqProcessorPanic, DialogsSnitch>,
    Arc<SipManager>,
) {
    let sip_manager = SipBuilder::new::<CorePanic, TransactionPanic, TransportSnitch>()
        .expect("sip manager failed")
        .manager;

    let core: Core<RegistrarSnitch, ReqProcessorPanic, DialogsSnitch> =
        Core::new(Arc::downgrade(&sip_manager));

    (core, sip_manager)
}

#[tokio::test]
async fn sending_an_unauthorized_register_request_receives_401() {
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
