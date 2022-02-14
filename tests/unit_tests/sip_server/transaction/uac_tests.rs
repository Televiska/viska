use crate::common::{
    advance_for,
    extensions::TransactionUacExt,
    factories::prelude::*,
    snitches::{CoreSnitch, TransportSnitch},
};
use common::futures_util::stream::StreamExt;
use common::log::Level;
use common::rsip::{self, prelude::*};
use models::transport::{RequestMsg, TransportMsg};
use sip_server::{
    transaction::uac::{TrxState, TrxStateMachine, TIMER_M},
    SipBuilder, SipManager, Transaction, TransactionLayer,
};
use std::any::Any;
use std::sync::Arc;
use std::time::Duration;

async fn setup() -> Arc<SipManager> {
    let builder =
        SipBuilder::new::<CoreSnitch, Transaction, TransportSnitch>().expect("sip manager failed");
    builder.run().await;

    builder.manager
}

#[tokio::test]
async fn if_peer_not_responding() {
    let sip_manager = setup().await;
    let transaction = sip_manager.transaction.clone();

    as_downcasted!(
        sip_manager,
        tu,
        transaction,
        transport,
        CoreSnitch,
        Transaction,
        TransportSnitch
    );

    assert_eq!(transport.messages.len().await, 0);

    let request: rsip::Request = requests::invite_request();
    let result = sip_manager
        .transaction
        .new_uac_invite_transaction(RequestMsg {
            sip_request: request.clone(),
            ..Randomized::default()
        })
        .await;
    assert!(result.is_ok(), "returns: {:?}", result);

    assert_eq!(transport.messages.len().await, 1);
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
    assert_eq!(transport.messages.len().await, 2);
    advance_for(Duration::from_millis(1000)).await;
    assert_eq!(transport.messages.len().await, 3);
    advance_for(Duration::from_millis(2000)).await;
    assert_eq!(transport.messages.len().await, 4);
    advance_for(Duration::from_millis(4000)).await;
    assert_eq!(transport.messages.len().await, 5);
    advance_for(Duration::from_millis(8000)).await;
    assert_eq!(transport.messages.len().await, 6);
    advance_for(Duration::from_millis(16000)).await;
    assert_eq!(transport.messages.len().await, 7);
    advance_for(Duration::from_millis(50000)).await;
    assert_eq!(transport.messages.len().await, 7);
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
    let sip_manager = setup().await;
    let transaction = sip_manager.transaction.clone();

    as_downcasted!(
        sip_manager,
        tu,
        transaction,
        transport,
        CoreSnitch,
        Transaction,
        TransportSnitch
    );

    assert_eq!(transport.messages.len().await, 0);

    let request: rsip::Request = requests::invite_request();
    let result = sip_manager
        .transaction
        .new_uac_invite_transaction(RequestMsg {
            sip_request: request.clone(),
            ..Randomized::default()
        })
        .await;
    assert!(result.is_ok(), "returns: {:?}", result);

    assert_eq!(transport.messages.len().await, 1);
    assert_eq!(tu.messages.len().await, 0);
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
    let result = sip_manager
        .transaction
        .process_incoming_message(TransportMsg {
            sip_message: response.clone().into(),
            ..Randomized::default()
        })
        .await;

    assert_eq!(transport.messages.len().await, 1);
    assert_eq!(tu.messages.len().await, 1);
    //assert_eq!(tu.messages.lock().await.len(), 1);

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
    let result = sip_manager
        .transaction
        .process_incoming_message(TransportMsg {
            sip_message: response.clone().into(),
            ..Randomized::default()
        })
        .await;

    assert_eq!(transport.messages.len().await, 1);
    assert_eq!(tu.messages.len().await, 2);
    //assert_eq!(tu.messages.lock().await.len(), 1);

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
    let sip_manager = setup().await;
    let transaction = sip_manager.transaction.clone();

    let transport = sip_manager.transport.clone();
    let transport = as_any!(transport, TransportSnitch);
    let tu = sip_manager.core.clone();
    let tu = as_any!(tu, CoreSnitch);
    let transaction = sip_manager.transaction.clone();
    let transaction = as_any!(transaction, Transaction);

    assert_eq!(transport.messages.len().await, 0);

    let request: rsip::Request = requests::invite_request();
    let result = sip_manager
        .transaction
        .new_uac_invite_transaction(RequestMsg {
            sip_request: request.clone(),
            ..Randomized::default()
        })
        .await;
    assert!(result.is_ok(), "result is error: {:?}", result);

    assert_eq!(transport.messages.len().await, 1);
    assert_eq!(tu.messages.len().await, 0);

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
    let result = sip_manager
        .transaction
        .process_incoming_message(TransportMsg {
            sip_message: response.clone().into(),
            ..Randomized::default()
        })
        .await;

    assert_eq!(transport.messages.len().await, 1);
    assert_eq!(tu.messages.len().await, 1);
    //assert_eq!(tu.messages.lock().await.len(), 1);

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
    let sip_manager = setup().await;
    let transaction = sip_manager.transaction.clone();

    let transport = sip_manager.transport.clone();
    let transport = as_any!(transport, TransportSnitch);
    let tu = sip_manager.core.clone();
    let tu = as_any!(tu, CoreSnitch);
    let transaction = sip_manager.transaction.clone();
    let transaction = as_any!(transaction, Transaction);

    assert_eq!(transport.messages.len().await, 0);

    let request: rsip::Request = requests::invite_request();
    let result = sip_manager
        .transaction
        .new_uac_invite_transaction(RequestMsg {
            sip_request: request.clone(),
            ..Randomized::default()
        })
        .await;
    assert!(result.is_ok(), "result is error: {:?}", result);

    assert_eq!(transport.messages.len().await, 1);
    assert_eq!(tu.messages.len().await, 0);

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
    let result = sip_manager
        .transaction
        .process_incoming_message(TransportMsg {
            sip_message: response.clone().into(),
            ..Randomized::default()
        })
        .await;

    assert_eq!(transport.messages.len().await, 1);
    assert_eq!(tu.messages.len().await, 1);
    //assert_eq!(tu.messages.lock().await.len(), 1);

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
    let result = sip_manager
        .transaction
        .process_incoming_message(TransportMsg {
            sip_message: response.clone().into(),
            ..Randomized::default()
        })
        .await;

    assert_eq!(transport.messages.len().await, 2);
    assert_eq!(tu.messages.len().await, 1);
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
    let sip_manager = setup().await;
    let transaction = sip_manager.transaction.clone();

    let transport = sip_manager.transport.clone();
    let transport = as_any!(transport, TransportSnitch);
    let tu = sip_manager.core.clone();
    let tu = as_any!(tu, CoreSnitch);
    let transaction = sip_manager.transaction.clone();
    let transaction = as_any!(transaction, Transaction);

    assert_eq!(transport.messages.len().await, 0);

    let request: rsip::Request = requests::invite_request();
    let result = sip_manager
        .transaction
        .new_uac_invite_transaction(RequestMsg {
            sip_request: request.clone(),
            ..Randomized::default()
        })
        .await;
    assert!(result.is_ok(), "result is error: {:?}", result);

    assert_eq!(transport.messages.len().await, 1);
    assert_eq!(tu.messages.len().await, 0);

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
    let result = sip_manager
        .transaction
        .process_incoming_message(TransportMsg {
            sip_message: response.clone().into(),
            ..Randomized::default()
        })
        .await;

    assert_eq!(transport.messages.len().await, 1);
    assert_eq!(tu.messages.len().await, 1);
    //assert_eq!(tu.messages.lock().await.len(), 1);

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
    let result = sip_manager
        .transaction
        .process_incoming_message(TransportMsg {
            sip_message: response.clone().into(),
            ..Randomized::default()
        })
        .await;

    assert_eq!(transport.messages.len().await, 1);
    assert_eq!(tu.messages.len().await, 2);
    //assert_eq!(tu.messages.lock().await.len(), 1);

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
    let result = sip_manager
        .transaction
        .process_incoming_message(TransportMsg {
            sip_message: response.clone().into(),
            ..Randomized::default()
        })
        .await;

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
    let sip_manager = setup().await;
    let transaction = sip_manager.transaction.clone();

    let transport = sip_manager.transport.clone();
    let transport = as_any!(transport, TransportSnitch);
    let tu = sip_manager.core.clone();
    let tu = as_any!(tu, CoreSnitch);
    let transaction = sip_manager.transaction.clone();
    let transaction = as_any!(transaction, Transaction);

    assert_eq!(transport.messages.len().await, 0);

    let request: rsip::Request = requests::invite_request();
    let result = sip_manager
        .transaction
        .new_uac_invite_transaction(RequestMsg {
            sip_request: request.clone(),
            ..Randomized::default()
        })
        .await;
    assert!(result.is_ok(), "result is error: {:?}", result);

    assert_eq!(transport.messages.len().await, 1);
    assert_eq!(tu.messages.len().await, 0);
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
    let result = sip_manager
        .transaction
        .process_incoming_message(TransportMsg {
            sip_message: response.clone().into(),
            ..Randomized::default()
        })
        .await;

    assert_eq!(transport.messages.len().await, 1);
    assert_eq!(tu.messages.len().await, 1);
    //assert_eq!(tu.messages.lock().await.len(), 1);

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
    let result = sip_manager
        .transaction
        .process_incoming_message(TransportMsg {
            sip_message: response.clone().into(),
            ..Randomized::default()
        })
        .await;

    assert_eq!(transport.messages.len().await, 2);
    assert_eq!(tu.messages.len().await, 2);
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
    let result = sip_manager
        .transaction
        .process_incoming_message(TransportMsg {
            sip_message: response.clone().into(),
            ..Randomized::default()
        })
        .await;

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
