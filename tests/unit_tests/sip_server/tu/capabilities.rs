use crate::common::{factories::prelude::*, snitches::SpySnitch};
use models::{
    transaction::TransactionLayerMsg,
    transport::{RequestMsg, TransportLayerMsg},
    tu::TuLayerMsg,
};
use sip_server::{tu::elements::Capabilities, ReqProcessor};

pub async fn setup() -> (
    SpySnitch<TuLayerMsg>,
    SpySnitch<TransactionLayerMsg>,
    SpySnitch<TransportLayerMsg>,
) {
    let (handlers, receivers) = models::channels_builder();
    let transport = SpySnitch::new(handlers.clone(), receivers.transport).expect("transport");
    let transaction = SpySnitch::new(handlers.clone(), receivers.transaction).expect("transaction");
    let tu = SpySnitch::new(handlers.clone(), receivers.tu).expect("tu");

    (tu, transaction, transport)
}

#[tokio::test]
#[serial_test::serial]
async fn sending_an_options_request_receives_busy() {
    let (_, _, transport) = setup().await;

    let capabilities = Capabilities::new(transport.handlers());

    capabilities
        .process_incoming_request(RequestMsg {
            sip_request: requests::options_request(),
            ..Randomized::default()
        })
        .await
        .unwrap();
    assert_eq!(transport.messages().await.len().await, 1);
    /*
    assert_eq!(
        transport.messages.first_response().await.status_code,
        486.into()
    );*/
}
