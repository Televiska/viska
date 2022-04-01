use crate::common::{factories::prelude::*, snitches::SpySnitch};
use common::rsip::{self, common::Uri, message::HeadersExt};
use models::{
    transaction::TransactionLayerMsg, transport::TransportLayerMsg, tu::TuLayerMsg, Handlers,
};
use sip_server::tu::dialogs::uac::dialog_sm::{DialogSm, DialogState};

pub async fn setup() -> (
    Handlers,
    (
        SpySnitch<TuLayerMsg>,
        SpySnitch<TransactionLayerMsg>,
        SpySnitch<TransportLayerMsg>,
    ),
) {
    let (handlers, receivers) = models::channels_builder();
    let transport = SpySnitch::new(handlers.clone(), receivers.transport).expect("transport");
    let transaction = SpySnitch::new(handlers.clone(), receivers.transaction).expect("transaction");
    let tu = SpySnitch::new(handlers.clone(), receivers.tu).expect("tu");

    (handlers, (tu, transaction, transport))
}

#[tokio::test]
async fn creates_unconfirmed_dialog_and_initializes_correctly() {
    let (handlers, (tu, transaction, transport)) = setup().await;

    let request: rsip::Request = requests::invite_request();
    let dialog_sm = DialogSm::new(handlers, request.clone()).await.unwrap();

    assert_eq!(transaction.messages().await.len().await, 1);
    assert_eq!(tu.messages().await.len().await, 0);
    assert_eq!(transport.messages().await.len().await, 0);
    assert!(matches!(dialog_sm.state, DialogState::Unconfirmed(..)));

    assert_eq!(
        dialog_sm.local_tag,
        request
            .clone()
            .from_header()
            .unwrap()
            .tag()
            .ok()
            .flatten()
            .unwrap()
    );
    assert_eq!(dialog_sm.local_seqn, 1);
    assert_eq!(
        dialog_sm.local_uri,
        request.from_header().unwrap().uri().unwrap()
    );
    assert_eq!(dialog_sm.call_id, *request.call_id_header().unwrap());
}

#[tokio::test]
async fn creates_a_confirmed_dialog() {
    let (handlers, (tu, transaction, transport)) = setup().await;

    let request = requests::invite_request();
    let mut dialog_sm = DialogSm::new(handlers, request.clone()).await.unwrap();

    assert_eq!(transaction.messages().await.len().await, 1);
    assert_eq!(tu.messages().await.len().await, 0);
    assert_eq!(transport.messages().await.len().await, 0);
    assert!(matches!(dialog_sm.state, DialogState::Unconfirmed(..)));

    dialog_sm
        .process_incoming_response(responses::ringing_response_from(request.clone()))
        .await;
    assert_eq!(transaction.messages().await.len().await, 1);
    assert_eq!(tu.messages().await.len().await, 0);
    assert_eq!(transport.messages().await.len().await, 0);
    assert!(matches!(dialog_sm.state, DialogState::Early(..)));

    let ok_response = responses::ok_response_from(request.clone());
    dialog_sm
        .process_incoming_response(ok_response.clone())
        .await;
    assert_eq!(transaction.messages().await.len().await, 1);
    assert_eq!(tu.messages().await.len().await, 0);
    assert_eq!(transport.messages().await.len().await, 1);

    assert!(matches!(dialog_sm.state, DialogState::Confirmed(..)));

    assert_eq!(
        dialog_sm.remote_tag,
        Some(ok_response.clone().to_header().unwrap().tag().ok().flatten().unwrap())
    );
    assert_eq!(
        dialog_sm.remote_seqn,
        Some(ok_response.clone().cseq_header().unwrap().seq().unwrap())
    );
    assert_eq!(
        dialog_sm.remote_target,
        Some(ok_response.clone().contact_header().unwrap().uri().unwrap())
    );

    let ack_message = transport
        .messages()
        .await
        .latest()
        .await
        .outgoing_sip_request();
    assert_eq!(ack_message.method, rsip::Method::Ack);
    assert_eq!(
        ack_message.cseq_header().unwrap().seq().unwrap(),
        request.cseq_header().unwrap().seq().unwrap()
    );
}

#[tokio::test]
async fn modifies_a_confirmed_dialog() {
    let (handlers, (tu, transaction, transport)) = setup().await;

    let mut request = requests::invite_request();
    let mut dialog_sm = DialogSm::new(handlers, request.clone()).await.unwrap();

    let ok_response = responses::ok_response_from(request.clone());
    dialog_sm
        .process_incoming_response(ok_response.clone())
        .await;
    assert_eq!(transaction.messages().await.len().await, 1);
    assert_eq!(tu.messages().await.len().await, 0);
    assert_eq!(transport.messages().await.len().await, 1);

    assert!(matches!(dialog_sm.state, DialogState::Confirmed(..)));

    let new_uri = Uri::default().sip().with_user("another");
    request
        .contact_header_mut()
        .unwrap()
        .mut_uri(new_uri.clone())
        .unwrap();
    dialog_sm.process_outgoing_request(request).await;
    assert_eq!(transaction.messages().await.len().await, 2);
    let invite_req = transaction
        .messages()
        .await
        .try_latest()
        .await
        .new_uac_invite_sip_msg();
    assert_eq!(invite_req.cseq_header().unwrap().seq().unwrap(), 2);
    assert_eq!(invite_req.contact_header().unwrap().uri().unwrap(), new_uri);
}
/*
#[tokio::test]
async fn peer_modifies_a_confirmed_dialog() {
    let (handlers, (tu, transaction, transport)) = setup().await;

    let mut request = requests::invite_request();
    let mut dialog_sm = DialogSm::new(handlers, request.clone()).await.unwrap();

    let ok_response = responses::ok_response_from(request.clone());
    dialog_sm
        .process_incoming_response(ok_response.clone())
        .await;
    assert_eq!(transaction.messages().await.len().await, 1);
    assert_eq!(tu.messages().await.len().await, 0);
    assert_eq!(transport.messages().await.len().await, 1);

    assert!(matches!(dialog_sm.state, DialogState::Confirmed(..)));

    let new_uri = Uri::default().sip().with_user("another");
    request
        .contact_header_mut()
        .unwrap()
        .mut_uri(new_uri.clone())
        .unwrap();
    dialog_sm.process_incoming_request(request).await;
    assert_eq!(transaction.messages().await.len().await, 2);
    let invite_req = transaction
        .messages()
        .await
        .try_latest()
        .await
        .new_uac_invite_sip_msg();
    assert_eq!(invite_req.cseq_header().unwrap().seq().unwrap(), 2);
    assert_eq!(invite_req.contact_header().unwrap().uri().unwrap(), new_uri);
}*/

#[tokio::test]
async fn closing_a_dialog() {
    let (handlers, (tu, transaction, transport)) = setup().await;

    let request = requests::invite_request();
    let mut dialog_sm = DialogSm::new(handlers, request.clone()).await.unwrap();

    let ok_response = responses::ok_response_from(request.clone());
    dialog_sm
        .process_incoming_response(ok_response.clone())
        .await;
    assert_eq!(transaction.messages().await.len().await, 1);
    assert_eq!(tu.messages().await.len().await, 0);
    assert_eq!(transport.messages().await.len().await, 1);

    assert!(matches!(dialog_sm.state, DialogState::Confirmed(..)));

    let bye = requests::bye_request();
    dialog_sm.process_outgoing_request(bye).await;
    assert_eq!(transaction.messages().await.len().await, 2);
    let invite_req = transaction
        .messages()
        .await
        .try_latest()
        .await
        .new_uac_sip_msg();
    assert_eq!(invite_req.cseq_header().unwrap().seq().unwrap(), 2);
    assert!(matches!(dialog_sm.state, DialogState::Terminated(..)));
}
