use super::setup;
use crate::common::{
    advance_for, extensions::transaction_ext::TransactionUacExt, factories::prelude::*,
};
use common::rsip::{self, prelude::*};
use sip_server::transaction::sm::uac::TIMER_K;
use std::time::Duration;

#[tokio::test]
async fn if_peer_not_responding() {
    let (_, transaction, transport) = setup().await;

    let request: rsip::Request = requests::bye_request();
    transaction
        .handler()
        .new_uac(request.clone())
        .await
        .unwrap();

    assert_eq!(transport.messages().await.len().await, 1);

    assert_eq!(transaction.inner.state.read().await.len(), 1);
    assert!(
        transaction
            .is_trying(
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
    assert_eq!(transaction.inner.state.read().await.len(), 1);
    assert!(
        transaction
            .is_timedout(
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

    let request: rsip::Request = requests::bye_request();
    let result = transaction.handler().new_uac(request.clone()).await;
    assert!(result.is_ok(), "returns: {:?}", result);

    assert_eq!(transport.messages().await.len().await, 1);
    assert_eq!(tu.messages().await.len().await, 0);
    assert_eq!(transaction.inner.state.read().await.len(), 1);
    assert!(
        transaction
            .is_trying(
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
        .process(response.clone().into())
        .await
        .unwrap();

    assert_eq!(transport.messages().await.len().await, 1);
    assert_eq!(tu.messages().await.len().await, 1);

    assert_eq!(transaction.inner.state.read().await.len(), 1);
    assert!(
        transaction
            .is_proceeding(
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
        .process(response.clone().into())
        .await
        .unwrap();

    assert_eq!(transport.messages().await.len().await, 1);
    assert_eq!(tu.messages().await.len().await, 2);

    assert_eq!(transaction.inner.state.read().await.len(), 1);
    assert!(
        transaction
            .is_completed(
                request
                    .transaction_id()
                    .unwrap()
                    .expect("response transaction id")
                    .into()
            )
            .await
    );

    advance_for(Duration::from_millis(TIMER_K)).await;

    assert!(
        transaction
            .is_terminated(
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

    let request: rsip::Request = requests::bye_request();
    transaction
        .handler()
        .new_uac(request.clone())
        .await
        .unwrap();

    assert_eq!(transport.messages().await.len().await, 1);
    assert_eq!(tu.messages().await.len().await, 0);

    assert_eq!(transaction.inner.state.read().await.len(), 1);
    assert!(
        transaction
            .is_trying(
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
        .process(response.clone().into())
        .await
        .unwrap();

    assert_eq!(transport.messages().await.len().await, 1);
    assert_eq!(tu.messages().await.len().await, 1);

    assert_eq!(transaction.inner.state.read().await.len(), 1);
    assert!(
        transaction
            .is_completed(
                request
                    .transaction_id()
                    .unwrap()
                    .expect("response transaction id")
                    .into()
            )
            .await
    );

    advance_for(Duration::from_millis(TIMER_K)).await;

    assert!(
        transaction
            .is_terminated(
                request
                    .transaction_id()
                    .unwrap()
                    .expect("response transaction id")
                    .into()
            )
            .await
    );
}
