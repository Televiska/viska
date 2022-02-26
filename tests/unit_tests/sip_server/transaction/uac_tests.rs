use crate::common::{
    advance_for, extensions::TransactionUacExt, factories::prelude::*, snitches::SpySnitch,
};
use common::rsip::{self, prelude::*};
use models::{
    transport::{RequestMsg, TransportLayerMsg, TransportMsg},
    tu::TuLayerMsg,
};
use sip_server::{transaction::uac::TIMER_M, Transaction};
use std::time::Duration;

async fn setup() -> (
    SpySnitch<TuLayerMsg>,
    Transaction,
    SpySnitch<TransportLayerMsg>,
) {
    let (handlers, receivers) = models::channels_builder();
    let transport = SpySnitch::new(handlers.clone(), receivers.transport).expect("transport");
    let transaction =
        Transaction::new(handlers.clone(), receivers.transaction).expect("transaction");
    let tu = SpySnitch::new(handlers.clone(), receivers.tu).expect("tu");

    (tu, transaction, transport)
}

#[tokio::test]
async fn if_peer_not_responding() {
    let (_, transaction, transport) = setup().await;

    let request: rsip::Request = requests::invite_request();
    let result = transaction
        .handler()
        .new_uac_invite(RequestMsg {
            sip_request: request.clone(),
            ..Randomized::default()
        })
        .await;
    assert!(result.is_ok(), "returns: {:?}", result);

    assert_eq!(transport.messages().await.len().await, 1);

    assert_eq!(transaction.inner.uac_state.read().await.len(), 1);
    assert!(
        transaction
            .is_uac_calling(
                request
                    .transaction_id()
                    .unwrap()
                    .expect("response transaction id")
                    .into()
            )
            .await
    );
    advance_for(Duration::from_millis(500)).await;
    assert_eq!(transport.messages().await.len().await, 2);
    advance_for(Duration::from_millis(1000)).await;
    assert_eq!(transport.messages().await.len().await, 3);
    advance_for(Duration::from_millis(2000)).await;
    assert_eq!(transport.messages().await.len().await, 4);
    advance_for(Duration::from_millis(4000)).await;
    assert_eq!(transport.messages().await.len().await, 5);
    advance_for(Duration::from_millis(8000)).await;
    assert_eq!(transport.messages().await.len().await, 6);
    advance_for(Duration::from_millis(16000)).await;
    assert_eq!(transport.messages().await.len().await, 7);
    advance_for(Duration::from_millis(50000)).await;
    assert_eq!(transport.messages().await.len().await, 7);
    assert_eq!(transaction.inner.uac_state.read().await.len(), 1);
    assert!(
        transaction
            .is_uac_terminated(
                request
                    .transaction_id()
                    .unwrap()
                    .expect("response transaction id")
                    .into()
            )
            .await
    );
}

#[tokio::test]
async fn with_trying_goes_through_proceeding() {
    let (tu, transaction, transport) = setup().await;

    let request: rsip::Request = requests::invite_request();
    let result = transaction
        .handler()
        .new_uac_invite(RequestMsg {
            sip_request: request.clone(),
            ..Randomized::default()
        })
        .await;
    assert!(result.is_ok(), "returns: {:?}", result);

    assert_eq!(transport.messages().await.len().await, 1);
    assert_eq!(tu.messages().await.len().await, 0);
    assert_eq!(transaction.inner.uac_state.read().await.len(), 1);
    assert!(
        transaction
            .is_uac_calling(
                request
                    .transaction_id()
                    .unwrap()
                    .expect("response transaction id")
                    .into()
            )
            .await
    );

    let response: rsip::Response = responses::trying_response_from(request.clone());
    transaction
        .handler()
        .process(TransportMsg {
            sip_message: response.clone().into(),
            ..Randomized::default()
        })
        .await
        .unwrap();

    assert_eq!(transport.messages().await.len().await, 1);
    assert_eq!(tu.messages().await.len().await, 1);

    assert_eq!(transaction.inner.uac_state.read().await.len(), 1);
    assert!(
        transaction
            .is_uac_proceeding(
                request
                    .transaction_id()
                    .unwrap()
                    .expect("response transaction id")
                    .into()
            )
            .await
    );

    let response: rsip::Response = responses::ok_response_from(request.clone());
    transaction
        .handler()
        .process(TransportMsg {
            sip_message: response.clone().into(),
            ..Randomized::default()
        })
        .await
        .unwrap();

    assert_eq!(transport.messages().await.len().await, 1);
    assert_eq!(tu.messages().await.len().await, 2);

    assert_eq!(transaction.inner.uac_state.read().await.len(), 1);
    assert!(
        transaction
            .is_uac_accepted(
                request
                    .transaction_id()
                    .unwrap()
                    .expect("response transaction id")
                    .into()
            )
            .await
    );

    advance_for(Duration::from_millis(TIMER_M)).await;

    assert!(
        transaction
            .is_uac_terminated(
                request
                    .transaction_id()
                    .unwrap()
                    .expect("response transaction id")
                    .into()
            )
            .await
    );
}

#[tokio::test]
async fn request_failure_goes_through_completed() {
    let (tu, transaction, transport) = setup().await;

    let request: rsip::Request = requests::invite_request();
    transaction
        .handler()
        .new_uac_invite(RequestMsg {
            sip_request: request.clone(),
            ..Randomized::default()
        })
        .await
        .unwrap();

    assert_eq!(transport.messages().await.len().await, 1);
    assert_eq!(tu.messages().await.len().await, 0);

    assert_eq!(transaction.inner.uac_state.read().await.len(), 1);
    assert!(
        transaction
            .is_uac_calling(
                request
                    .transaction_id()
                    .unwrap()
                    .expect("response transaction id")
                    .into()
            )
            .await
    );

    let response: rsip::Response = responses::request_failure_response_from(request.clone());
    transaction
        .handler()
        .process(TransportMsg {
            sip_message: response.clone().into(),
            ..Randomized::default()
        })
        .await
        .unwrap();

    assert_eq!(transport.messages().await.len().await, 1);
    assert_eq!(tu.messages().await.len().await, 1);

    assert_eq!(transaction.inner.uac_state.read().await.len(), 1);
    assert!(
        transaction
            .is_uac_completed(
                request
                    .transaction_id()
                    .unwrap()
                    .expect("response transaction id")
                    .into()
            )
            .await
    );

    advance_for(Duration::from_millis(TIMER_M)).await;

    assert!(
        transaction
            .is_uac_terminated(
                request
                    .transaction_id()
                    .unwrap()
                    .expect("response transaction id")
                    .into()
            )
            .await
    );
}

#[tokio::test]
async fn multiple_request_failure_goes_through_completed() {
    let (tu, transaction, transport) = setup().await;

    let request: rsip::Request = requests::invite_request();
    transaction
        .handler()
        .new_uac_invite(RequestMsg {
            sip_request: request.clone(),
            ..Randomized::default()
        })
        .await
        .unwrap();

    assert_eq!(transport.messages().await.len().await, 1);
    assert_eq!(tu.messages().await.len().await, 0);

    assert_eq!(transaction.inner.uac_state.read().await.len(), 1);
    assert!(
        transaction
            .is_uac_calling(
                request
                    .transaction_id()
                    .unwrap()
                    .expect("response transaction id")
                    .into()
            )
            .await
    );

    let response: rsip::Response = responses::request_failure_response_from(request.clone());
    transaction
        .handler()
        .process(TransportMsg {
            sip_message: response.clone().into(),
            ..Randomized::default()
        })
        .await
        .unwrap();

    assert_eq!(transport.messages().await.len().await, 1);
    assert_eq!(tu.messages().await.len().await, 1);

    assert_eq!(transaction.inner.uac_state.read().await.len(), 1);
    assert!(
        transaction
            .is_uac_completed(
                request
                    .transaction_id()
                    .unwrap()
                    .expect("response transaction id")
                    .into()
            )
            .await
    );

    advance_for(Duration::from_millis(10)).await;

    let response: rsip::Response = responses::request_failure_response_from(request.clone());
    transaction
        .handler()
        .process(TransportMsg {
            sip_message: response.clone().into(),
            ..Randomized::default()
        })
        .await
        .unwrap();

    assert_eq!(transport.messages().await.len().await, 2);
    assert_eq!(tu.messages().await.len().await, 1);

    assert_eq!(transaction.inner.uac_state.read().await.len(), 1);
    assert!(
        transaction
            .is_uac_completed(
                response
                    .transaction_id()
                    .unwrap()
                    .expect("response transaction id")
                    .into()
            )
            .await
    );

    advance_for(Duration::from_millis(TIMER_M)).await;

    assert!(
        transaction
            .is_uac_terminated(
                response
                    .transaction_id()
                    .unwrap()
                    .expect("response transaction id")
                    .into()
            )
            .await
    );
}

#[tokio::test]
async fn unexpected_failures_when_accepted_goes_to_errored() {
    let (tu, transaction, transport) = setup().await;

    let request: rsip::Request = requests::invite_request();
    transaction
        .handler()
        .new_uac_invite(RequestMsg {
            sip_request: request.clone(),
            ..Randomized::default()
        })
        .await
        .unwrap();

    assert_eq!(transport.messages().await.len().await, 1);
    assert_eq!(tu.messages().await.len().await, 0);

    assert_eq!(transaction.inner.uac_state.read().await.len(), 1);
    assert!(
        transaction
            .is_uac_calling(
                request
                    .transaction_id()
                    .unwrap()
                    .expect("response transaction id")
                    .into()
            )
            .await
    );

    let response: rsip::Response = responses::trying_response_from(request.clone());
    transaction
        .handler()
        .process(TransportMsg {
            sip_message: response.clone().into(),
            ..Randomized::default()
        })
        .await
        .unwrap();

    assert_eq!(transport.messages().await.len().await, 1);
    assert_eq!(tu.messages().await.len().await, 1);

    assert_eq!(transaction.inner.uac_state.read().await.len(), 1);
    assert!(
        transaction
            .is_uac_proceeding(
                response
                    .transaction_id()
                    .unwrap()
                    .expect("response transaction id")
                    .into()
            )
            .await
    );

    let response: rsip::Response = responses::ok_response_from(request.clone());
    transaction
        .handler()
        .process(TransportMsg {
            sip_message: response.clone().into(),
            ..Randomized::default()
        })
        .await
        .unwrap();

    assert_eq!(transport.messages().await.len().await, 1);
    assert_eq!(tu.messages().await.len().await, 2);

    assert_eq!(transaction.inner.uac_state.read().await.len(), 1);
    assert!(
        transaction
            .is_uac_accepted(
                response
                    .transaction_id()
                    .unwrap()
                    .expect("response transaction id")
                    .into()
            )
            .await
    );

    advance_for(Duration::from_secs(1)).await;
    let response: rsip::Response = responses::request_failure_response_from(request.clone());
    transaction
        .handler()
        .process(TransportMsg {
            sip_message: response.clone().into(),
            ..Randomized::default()
        })
        .await
        .unwrap();

    assert!(
        transaction
            .is_uac_errored(
                response
                    .transaction_id()
                    .unwrap()
                    .expect("response transaction id")
                    .into()
            )
            .await
    );
}

#[tokio::test]
async fn ok_when_completed_goes_to_errored() {
    let (tu, transaction, transport) = setup().await;

    let request: rsip::Request = requests::invite_request();
    transaction
        .handler()
        .new_uac_invite(RequestMsg {
            sip_request: request.clone(),
            ..Randomized::default()
        })
        .await
        .unwrap();

    assert_eq!(transport.messages().await.len().await, 1);
    assert_eq!(tu.messages().await.len().await, 0);
    assert_eq!(transaction.inner.uac_state.read().await.len(), 1);
    assert!(
        transaction
            .is_uac_calling(
                request
                    .transaction_id()
                    .unwrap()
                    .expect("response transaction id")
                    .into()
            )
            .await
    );

    let response: rsip::Response = responses::trying_response_from(request.clone());
    transaction
        .handler()
        .process(TransportMsg {
            sip_message: response.clone().into(),
            ..Randomized::default()
        })
        .await
        .unwrap();

    assert_eq!(transport.messages().await.len().await, 1);
    assert_eq!(tu.messages().await.len().await, 1);

    assert_eq!(transaction.inner.uac_state.read().await.len(), 1);
    assert!(
        transaction
            .is_uac_proceeding(
                response
                    .transaction_id()
                    .unwrap()
                    .expect("response transaction id")
                    .into()
            )
            .await
    );

    let response: rsip::Response = responses::request_failure_response_from(request.clone());
    transaction
        .handler()
        .process(TransportMsg {
            sip_message: response.clone().into(),
            ..Randomized::default()
        })
        .await
        .unwrap();

    assert_eq!(transport.messages().await.len().await, 2);
    assert_eq!(tu.messages().await.len().await, 2);
    //assert_eq!(tu.messages.lock().await.len(), 1);

    assert_eq!(transaction.inner.uac_state.read().await.len(), 1);
    assert!(
        transaction
            .is_uac_completed(
                response
                    .transaction_id()
                    .unwrap()
                    .expect("response transaction id")
                    .into()
            )
            .await
    );

    advance_for(Duration::from_secs(1)).await;
    let response: rsip::Response = responses::ok_response_from(request.clone());
    transaction
        .handler()
        .process(TransportMsg {
            sip_message: response.clone().into(),
            ..Randomized::default()
        })
        .await
        .unwrap();

    assert!(
        transaction
            .is_uac_errored(
                response
                    .transaction_id()
                    .unwrap()
                    .expect("response transaction id")
                    .into()
            )
            .await
    );
}
