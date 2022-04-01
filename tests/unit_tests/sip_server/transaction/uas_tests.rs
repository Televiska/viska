use super::setup;
use crate::common::{advance_for, extensions::TransactionUasExt, factories::prelude::*};
use common::rsip::{self, prelude::*};
use models::{
    rsip_ext::*,
    transport::{RequestMsg, ResponseMsg, TransportLayerMsg, TransportMsg},
};
use std::time::Duration;

/* ##### proceeding state ##### */

#[tokio::test]
async fn if_peer_not_alive() {
    let (_, transaction, transport) = setup().await;

    let request: rsip::Request = requests::invite_request();
    transaction
        .handler()
        .new_uas_invite(
            RequestMsg {
                sip_request: request.clone(),
                ..Randomized::default()
            },
            None,
        )
        .await
        .unwrap();

    assert_eq!(transport.messages().await.len().await, 1);
    assert_eq!(transaction.inner.state.read().await.len(), 1);

    transaction
        .handler()
        .transport_error(
            RequestMsg {
                sip_request: request.clone(),
                ..Randomized::default()
            }
            .into(),
            "some error".into(),
        )
        .await
        .unwrap();

    assert_eq!(transport.messages().await.len().await, 1);
    assert!(
        transaction
            .is_uas_errored(
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
async fn multiple_invite_on_proceeding() {
    let (_, transaction, transport) = setup().await;

    let request: rsip::Request = requests::invite_request();
    transaction
        .handler()
        .new_uas_invite(
            RequestMsg {
                sip_request: request.clone(),
                ..Randomized::default()
            },
            Some(request.provisional_of(180)),
        )
        .await
        .unwrap();

    assert_eq!(transport.messages().await.len().await, 1);
    assert_eq!(transaction.inner.state.read().await.len(), 1);

    transaction
        .handler()
        .process(
            RequestMsg {
                sip_request: request.clone(),
                ..Randomized::default()
            }
            .into(),
        )
        .await
        .unwrap();
    assert_eq!(transport.messages().await.len().await, 2);
    match transport.messages().await.last().await {
        TransportLayerMsg::Outgoing(TransportMsg {
            sip_message:
                rsip::SipMessage::Response(rsip::Response {
                    status_code: rsip::StatusCode::Ringing,
                    ..
                }),
            peer: _,
            transport: _,
        }) => (),
        _ => panic!("unexpected message state"),
    };
    assert_eq!(transaction.inner.state.read().await.len(), 1);
}

#[tokio::test]
async fn with_redirect_response_moves_to_completed() {
    let (_, transaction, transport) = setup().await;

    let request: rsip::Request = requests::invite_request();
    transaction
        .handler()
        .new_uas_invite(
            RequestMsg {
                sip_request: request.clone(),
                ..Randomized::default()
            },
            Some(request.provisional_of(180)),
        )
        .await
        .unwrap();

    assert_eq!(transport.messages().await.len().await, 1);
    assert_eq!(transaction.inner.state.read().await.len(), 1);

    transaction
        .handler()
        .reply(ResponseMsg {
            sip_response: responses::redirection_response_from(request.clone()),
            ..Randomized::default()
        })
        .await
        .unwrap();
    assert!(
        transaction
            .is_uas_completed(
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
async fn with_ok_response_moves_to_accepted() {
    let (_, transaction, transport) = setup().await;

    let request: rsip::Request = requests::invite_request();
    transaction
        .handler()
        .new_uas_invite(
            RequestMsg {
                sip_request: request.clone(),
                ..Randomized::default()
            },
            Some(request.provisional_of(180)),
        )
        .await
        .unwrap();

    assert_eq!(transport.messages().await.len().await, 1);
    assert_eq!(transaction.inner.state.read().await.len(), 1);

    transaction
        .handler()
        .reply(ResponseMsg {
            sip_response: responses::ok_response_from(request.clone()),
            ..Randomized::default()
        })
        .await
        .unwrap();
    assert!(
        transaction
            .is_uas_accepted(
                request
                    .transaction_id()
                    .unwrap()
                    .expect("response transaction id")
                    .into()
            )
            .await
    );
}

/* ##### completed state ##### */
#[tokio::test]
async fn multiple_invites_on_completed_resends_response() {
    let (_, transaction, transport) = setup().await;

    let request: rsip::Request = requests::invite_request();
    transaction
        .handler()
        .new_uas_invite(
            RequestMsg {
                sip_request: request.clone(),
                ..Randomized::default()
            },
            Some(request.provisional_of(180)),
        )
        .await
        .unwrap();

    let response = responses::redirection_response_from(request.clone());
    transaction
        .handler()
        .reply(ResponseMsg {
            sip_response: response.clone(),
            ..Randomized::default()
        })
        .await
        .unwrap();

    assert!(
        transaction
            .is_uas_completed(
                request
                    .transaction_id()
                    .unwrap()
                    .expect("response transaction id")
                    .into()
            )
            .await
    );

    transaction
        .handler()
        .process(
            RequestMsg {
                sip_request: request.clone(),
                ..Randomized::default()
            }
            .into(),
        )
        .await
        .unwrap();
    assert_eq!(transport.messages().await.len().await, 3);
    match transport.messages().await.last().await {
        TransportLayerMsg::Outgoing(TransportMsg {
            sip_message: rsip::SipMessage::Response(resp),
            peer: _,
            transport: _,
        }) if resp == response => (),
        _ => panic!("unexpected message state"),
    };
    assert_eq!(transaction.inner.state.read().await.len(), 1);
}

#[tokio::test]
async fn redirect_but_peer_not_responding_with_ack() {
    let (_, transaction, transport) = setup().await;

    let request: rsip::Request = requests::invite_request();
    transaction
        .handler()
        .new_uas_invite(
            RequestMsg {
                sip_request: request.clone(),
                ..Randomized::default()
            },
            Some(request.provisional_of(180)),
        )
        .await
        .unwrap();

    let response = responses::redirection_response_from(request.clone());
    transaction
        .handler()
        .reply(ResponseMsg {
            sip_response: response.clone(),
            ..Randomized::default()
        })
        .await
        .unwrap();

    assert!(
        transaction
            .is_uas_completed(
                request
                    .transaction_id()
                    .unwrap()
                    .expect("response transaction id")
                    .into()
            )
            .await
    );
    assert_eq!(transport.messages().await.len().await, 2);

    advance_for(Duration::from_millis(500)).await;
    assert_eq!(transport.messages().await.len().await, 3);
    advance_for(Duration::from_millis(1000)).await;
    assert_eq!(transport.messages().await.len().await, 4);
    advance_for(Duration::from_millis(2000)).await; //3.5
    assert_eq!(transport.messages().await.len().await, 5);
    advance_for(Duration::from_millis(4000)).await; //7.5
    assert_eq!(transport.messages().await.len().await, 6);
    advance_for(Duration::from_millis(4000)).await; //11.5
    assert_eq!(transport.messages().await.len().await, 7);
    advance_for(Duration::from_millis(4000)).await; //15.5
    assert_eq!(transport.messages().await.len().await, 8);
    advance_for(Duration::from_millis(4000)).await; //19.5
    assert_eq!(transport.messages().await.len().await, 9);
    advance_for(Duration::from_millis(4000)).await; //23.5
    assert_eq!(transport.messages().await.len().await, 10);
    advance_for(Duration::from_millis(4000)).await; //27.5
    assert_eq!(transport.messages().await.len().await, 11);
    advance_for(Duration::from_millis(4000)).await; //31.5
    assert_eq!(transport.messages().await.len().await, 12);
    //forward time H and check messages + error state
    advance_for(Duration::from_millis(4000)).await; //35.5
    assert_eq!(transport.messages().await.len().await, 12);

    assert!(
        transaction
            .is_uas_terminated(
                request
                    .transaction_id()
                    .unwrap()
                    .expect("request transaction id")
                    .into()
            )
            .await
    );
}

#[tokio::test]
async fn with_ack_moves_to_confirmed() {
    let (tu, transaction, transport) = setup().await;

    let request: rsip::Request = requests::invite_request();
    transaction
        .handler()
        .new_uas_invite(
            RequestMsg {
                sip_request: request.clone(),
                ..Randomized::default()
            },
            Some(request.provisional_of(180)),
        )
        .await
        .unwrap();

    let response = ResponseMsg {
        sip_response: responses::redirection_response_from(request.clone()),
        ..Randomized::default()
    };
    transaction.handler().reply(response.clone()).await.unwrap();
    assert!(
        transaction
            .is_uas_completed(
                request
                    .transaction_id()
                    .unwrap()
                    .expect("response transaction id")
                    .into()
            )
            .await
    );
    assert_eq!(tu.messages().await.len().await, 0);
    assert_eq!(transport.messages().await.len().await, 2);
    assert_eq!(transaction.inner.state.read().await.len(), 1);

    transaction
        .handler()
        .process(
            RequestMsg {
                sip_request: request.ack_request_from(response.sip_response),
                ..Randomized::default()
            }
            .into(),
        )
        .await
        .unwrap();
    assert_eq!(tu.messages().await.len().await, 0);
    assert_eq!(transport.messages().await.len().await, 2);
    assert_eq!(transaction.inner.state.read().await.len(), 1);
    assert!(
        transaction
            .is_uas_confirmed(
                request
                    .transaction_id()
                    .unwrap()
                    .expect("response transaction id")
                    .into()
            )
            .await
    );
}

/* ##### accepted state ##### */
#[tokio::test]
async fn multiple_invites_on_accepted_resends_response() {
    let (_, transaction, transport) = setup().await;

    let request: rsip::Request = requests::invite_request();
    transaction
        .handler()
        .new_uas_invite(
            RequestMsg {
                sip_request: request.clone(),
                ..Randomized::default()
            },
            Some(request.provisional_of(180)),
        )
        .await
        .unwrap();

    let response = responses::ok_response_from(request.clone());
    transaction
        .handler()
        .reply(ResponseMsg {
            sip_response: response.clone(),
            ..Randomized::default()
        })
        .await
        .unwrap();

    assert!(
        transaction
            .is_uas_accepted(
                request
                    .transaction_id()
                    .unwrap()
                    .expect("response transaction id")
                    .into()
            )
            .await
    );

    transaction
        .handler()
        .process(
            RequestMsg {
                sip_request: request.clone(),
                ..Randomized::default()
            }
            .into(),
        )
        .await
        .unwrap();
    assert_eq!(transport.messages().await.len().await, 3);
    match transport.messages().await.last().await {
        TransportLayerMsg::Outgoing(TransportMsg {
            sip_message: rsip::SipMessage::Response(resp),
            peer: _,
            transport: _,
        }) if resp == response => (),
        _ => panic!("unexpected message state"),
    };
    assert_eq!(transaction.inner.state.read().await.len(), 1);
}

#[tokio::test]
async fn ok_but_peer_not_responding_with_ack() {
    let (_, transaction, transport) = setup().await;

    let request: rsip::Request = requests::invite_request();
    transaction
        .handler()
        .new_uas_invite(
            RequestMsg {
                sip_request: request.clone(),
                ..Randomized::default()
            },
            Some(request.provisional_of(180)),
        )
        .await
        .unwrap();

    let response = responses::ok_response_from(request.clone());
    transaction
        .handler()
        .reply(ResponseMsg {
            sip_response: response.clone(),
            ..Randomized::default()
        })
        .await
        .unwrap();

    assert!(
        transaction
            .is_uas_accepted(
                request
                    .transaction_id()
                    .unwrap()
                    .expect("response transaction id")
                    .into()
            )
            .await
    );
    assert_eq!(transport.messages().await.len().await, 2);

    advance_for(Duration::from_millis(1000)).await;
    assert_eq!(transport.messages().await.len().await, 2);

    assert!(
        transaction
            .is_uas_accepted(
                request
                    .transaction_id()
                    .unwrap()
                    .expect("request transaction id")
                    .into()
            )
            .await
    );

    advance_for(Duration::from_millis(62 * 1000)).await;
    assert_eq!(transport.messages().await.len().await, 2);

    assert!(
        transaction
            .is_uas_terminated(
                request
                    .transaction_id()
                    .unwrap()
                    .expect("request transaction id")
                    .into()
            )
            .await
    );
}

#[tokio::test]
async fn with_multiple_ok_on_accepted() {
    let (_, transaction, transport) = setup().await;

    let request: rsip::Request = requests::invite_request();
    transaction
        .handler()
        .new_uas_invite(
            RequestMsg {
                sip_request: request.clone(),
                ..Randomized::default()
            },
            Some(request.provisional_of(180)),
        )
        .await
        .unwrap();

    assert_eq!(transport.messages().await.len().await, 1);
    assert_eq!(transaction.inner.state.read().await.len(), 1);

    transaction
        .handler()
        .reply(ResponseMsg {
            sip_response: responses::ok_response_from(request.clone()),
            ..Randomized::default()
        })
        .await
        .unwrap();
    assert!(
        transaction
            .is_uas_accepted(
                request
                    .transaction_id()
                    .unwrap()
                    .expect("response transaction id")
                    .into()
            )
            .await
    );

    transaction
        .handler()
        .reply(ResponseMsg {
            sip_response: responses::ok_response_from(request.clone()),
            ..Randomized::default()
        })
        .await
        .unwrap();

    assert_eq!(transport.messages().await.len().await, 3);
    assert_eq!(transaction.inner.state.read().await.len(), 1);
}

#[tokio::test]
async fn with_error_on_second_ok_on_accepted() {
    let (_, transaction, transport) = setup().await;

    let request: rsip::Request = requests::invite_request();
    transaction
        .handler()
        .new_uas_invite(
            RequestMsg {
                sip_request: request.clone(),
                ..Randomized::default()
            },
            Some(request.provisional_of(180)),
        )
        .await
        .unwrap();

    assert_eq!(transport.messages().await.len().await, 1);
    assert_eq!(transaction.inner.state.read().await.len(), 1);

    transaction
        .handler()
        .reply(ResponseMsg {
            sip_response: responses::ok_response_from(request.clone()),
            ..Randomized::default()
        })
        .await
        .unwrap();
    assert!(
        transaction
            .is_uas_accepted(
                request
                    .transaction_id()
                    .unwrap()
                    .expect("response transaction id")
                    .into()
            )
            .await
    );

    transaction
        .handler()
        .reply(ResponseMsg {
            sip_response: responses::ok_response_from(request.clone()),
            ..Randomized::default()
        })
        .await
        .unwrap();

    assert_eq!(transport.messages().await.len().await, 3);
    assert_eq!(transaction.inner.state.read().await.len(), 1);
    assert!(
        transaction
            .is_uas_accepted(
                request
                    .transaction_id()
                    .unwrap()
                    .expect("response transaction id")
                    .into()
            )
            .await
    );

    transaction
        .handler()
        .transport_error(
            RequestMsg {
                sip_request: request.clone(),
                ..Randomized::default()
            }
            .into(),
            "some error".into(),
        )
        .await
        .unwrap();

    assert_eq!(transport.messages().await.len().await, 3);
    assert_eq!(transaction.inner.state.read().await.len(), 1);

    assert!(
        transaction
            .is_uas_errored(
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
async fn multiple_ack_received_are_forwarded_to_tu() {
    let (tu, transaction, transport) = setup().await;

    let request: rsip::Request = requests::invite_request();
    transaction
        .handler()
        .new_uas_invite(
            RequestMsg {
                sip_request: request.clone(),
                ..Randomized::default()
            },
            Some(request.provisional_of(180)),
        )
        .await
        .unwrap();

    let response = ResponseMsg {
        sip_response: responses::ok_response_from(request.clone()),
        ..Randomized::default()
    };
    transaction.handler().reply(response.clone()).await.unwrap();
    assert!(
        transaction
            .is_uas_accepted(
                request
                    .transaction_id()
                    .unwrap()
                    .expect("response transaction id")
                    .into()
            )
            .await
    );
    assert_eq!(tu.messages().await.len().await, 0);
    assert_eq!(transport.messages().await.len().await, 2);
    assert_eq!(transaction.inner.state.read().await.len(), 1);

    transaction
        .handler()
        .process(
            RequestMsg {
                sip_request: request.ack_request_from(response.sip_response),
                ..Randomized::default()
            }
            .into(),
        )
        .await
        .unwrap();
    assert_eq!(tu.messages().await.len().await, 1);
    assert_eq!(transport.messages().await.len().await, 2);
    assert_eq!(transaction.inner.state.read().await.len(), 1);
}

/* ##### confirmed state ##### */
#[tokio::test]
async fn when_confirmed_acks_have_no_effect() {
    let (tu, transaction, transport) = setup().await;

    let request: rsip::Request = requests::invite_request();
    transaction
        .handler()
        .new_uas_invite(
            RequestMsg {
                sip_request: request.clone(),
                ..Randomized::default()
            },
            Some(request.provisional_of(180)),
        )
        .await
        .unwrap();

    let response = ResponseMsg {
        sip_response: responses::redirection_response_from(request.clone()),
        ..Randomized::default()
    };
    transaction.handler().reply(response.clone()).await.unwrap();

    transaction
        .handler()
        .process(
            RequestMsg {
                sip_request: request.ack_request_from(response.sip_response.clone()),
                ..Randomized::default()
            }
            .into(),
        )
        .await
        .unwrap();

    assert_eq!(tu.messages().await.len().await, 0);
    assert_eq!(transport.messages().await.len().await, 2);
    assert_eq!(transaction.inner.state.read().await.len(), 1);
    assert!(
        transaction
            .is_uas_confirmed(
                request
                    .transaction_id()
                    .unwrap()
                    .expect("response transaction id")
                    .into()
            )
            .await
    );

    transaction
        .handler()
        .process(
            RequestMsg {
                sip_request: request.ack_request_from(response.sip_response.clone()),
                ..Randomized::default()
            }
            .into(),
        )
        .await
        .unwrap();
    transaction
        .handler()
        .process(
            RequestMsg {
                sip_request: request.ack_request_from(response.sip_response),
                ..Randomized::default()
            }
            .into(),
        )
        .await
        .unwrap();

    assert_eq!(tu.messages().await.len().await, 0);
    assert_eq!(transport.messages().await.len().await, 2);
    assert_eq!(transaction.inner.state.read().await.len(), 1);
    assert!(
        transaction
            .is_uas_confirmed(
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
async fn when_confirmed_when_time_i_kicks_in_move_to_terminated() {
    let (tu, transaction, transport) = setup().await;

    let request: rsip::Request = requests::invite_request();
    transaction
        .handler()
        .new_uas_invite(
            RequestMsg {
                sip_request: request.clone(),
                ..Randomized::default()
            },
            Some(request.provisional_of(180)),
        )
        .await
        .unwrap();

    let response = ResponseMsg {
        sip_response: responses::redirection_response_from(request.clone()),
        ..Randomized::default()
    };
    transaction.handler().reply(response.clone()).await.unwrap();

    transaction
        .handler()
        .process(
            RequestMsg {
                sip_request: request.ack_request_from(response.sip_response.clone()),
                ..Randomized::default()
            }
            .into(),
        )
        .await
        .unwrap();

    assert_eq!(tu.messages().await.len().await, 0);
    assert_eq!(transport.messages().await.len().await, 2);
    assert_eq!(transaction.inner.state.read().await.len(), 1);
    assert!(
        transaction
            .is_uas_confirmed(
                request
                    .transaction_id()
                    .unwrap()
                    .expect("response transaction id")
                    .into()
            )
            .await
    );

    advance_for(Duration::from_millis(5000)).await;

    assert_eq!(tu.messages().await.len().await, 0);
    assert_eq!(transport.messages().await.len().await, 2);
    assert_eq!(transaction.inner.state.read().await.len(), 1);
    assert!(
        transaction
            .is_uas_terminated(
                request
                    .transaction_id()
                    .unwrap()
                    .expect("response transaction id")
                    .into()
            )
            .await
    );
}
