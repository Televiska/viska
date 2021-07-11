use crate::common::{
    advance_for,
    extensions::TransactionUasExt,
    factories::prelude::*,
    snitches::{CoreSnitch, TransportErrorSnitch, TransportSnitch},
};
use common::futures_util::stream::StreamExt;
use common::log::Level;
use common::rsip::prelude::*;
use models::{
    transport::{RequestMsg, ResponseMsg, TransportMsg},
    RequestExt,
};
use sip_server::{
    transaction::uas::{TrxState, TrxStateMachine, TIMER_G},
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

async fn setup_with_error_transport() -> Arc<SipManager> {
    let builder = SipBuilder::new::<CoreSnitch, Transaction, TransportErrorSnitch>()
        .expect("sip manager failed");
    builder.run().await;

    builder.manager
}

/* ##### proceeding state ##### */

#[tokio::test]
async fn if_peer_not_alive() {
    let sip_manager = setup_with_error_transport().await;
    let transaction = sip_manager.transaction.clone();

    as_downcasted!(
        sip_manager,
        core,
        transaction,
        transport,
        CoreSnitch,
        Transaction,
        TransportErrorSnitch
    );

    assert_eq!(transport.messages.len().await, 0);

    let request: rsip::Request = requests::invite_request();
    let result = sip_manager
        .transaction
        .new_uas_invite_transaction(
            RequestMsg {
                sip_request: request.clone(),
                ..Randomized::default()
            },
            None,
        )
        .await;
    assert!(result.is_err());

    assert_eq!(transport.messages.len().await, 0);
    assert_eq!(transaction.inner.uas_state.read().await.len(), 0);
}

#[tokio::test]
async fn transport_errors_on_second_provisional() {
    let sip_manager = setup_with_error_transport().await;
    let transaction = sip_manager.transaction.clone();

    as_downcasted!(
        sip_manager,
        core,
        transaction,
        transport,
        CoreSnitch,
        Transaction,
        TransportErrorSnitch
    );
    transport.turn_fail_switch_off().await;

    assert_eq!(transport.messages.len().await, 0);

    let request: rsip::Request = requests::invite_request();
    let result = transaction
        .new_uas_invite_transaction(
            RequestMsg {
                sip_request: request.clone(),
                ..Randomized::default()
            },
            None,
        )
        .await;
    assert!(result.is_ok());

    assert_eq!(transport.messages.len().await, 1);
    assert_eq!(transaction.inner.uas_state.read().await.len(), 1);
    //TODO: check state here

    transport.turn_fail_switch_on().await;
    let result = transaction
        .send(ResponseMsg {
            sip_response: request.provisional_of(180),
            ..Randomized::default()
        })
        .await;
    assert!(result.is_err());
    assert!(
        transaction
            .is_uas_errored(request.transaction_id().expect("response transaction id"))
            .await
    );
}

#[tokio::test]
async fn multiple_invite_on_proceeding() {
    let sip_manager = setup().await;
    let transaction = sip_manager.transaction.clone();

    as_downcasted!(
        sip_manager,
        core,
        transaction,
        transport,
        CoreSnitch,
        Transaction,
        TransportSnitch
    );

    assert_eq!(transport.messages.len().await, 0);

    let request: rsip::Request = requests::invite_request();
    let result = transaction
        .new_uas_invite_transaction(
            RequestMsg {
                sip_request: request.clone(),
                ..Randomized::default()
            },
            Some(request.provisional_of(180)),
        )
        .await;
    assert!(result.is_ok());

    assert_eq!(transport.messages.len().await, 1);
    assert_eq!(transaction.inner.uas_state.read().await.len(), 1);

    let result = transaction
        .process_incoming_message(
            RequestMsg {
                sip_request: request.clone(),
                ..Randomized::default()
            }
            .into(),
        )
        .await;
    assert_eq!(transport.messages.len().await, 2);
    match transport.messages.last().await {
        TransportMsg {
            sip_message:
                rsip::SipMessage::Response(rsip::Response {
                    status_code: rsip::common::StatusCode::Ringing,
                    ..
                }),
            peer: _,
            transport: _,
        } => (),
        _ => panic!("unexpected message state"),
    };
    assert_eq!(transaction.inner.uas_state.read().await.len(), 1);
}

#[tokio::test]
async fn with_redirect_response_moves_to_completed() {
    let sip_manager = setup().await;
    let transaction = sip_manager.transaction.clone();

    as_downcasted!(
        sip_manager,
        core,
        transaction,
        transport,
        CoreSnitch,
        Transaction,
        TransportSnitch
    );

    assert_eq!(transport.messages.len().await, 0);

    let request: rsip::Request = requests::invite_request();
    let result = transaction
        .new_uas_invite_transaction(
            RequestMsg {
                sip_request: request.clone(),
                ..Randomized::default()
            },
            Some(request.provisional_of(180)),
        )
        .await;
    assert!(result.is_ok());

    assert_eq!(transport.messages.len().await, 1);
    assert_eq!(transaction.inner.uas_state.read().await.len(), 1);

    let result = transaction
        .send(ResponseMsg {
            sip_response: responses::redirection_response_from(request.clone()),
            ..Randomized::default()
        })
        .await;
    assert!(result.is_ok());
    assert!(
        transaction
            .is_uas_completed(request.transaction_id().expect("response transaction id"))
            .await
    );
}

#[tokio::test]
async fn with_ok_response_moves_to_accepted() {
    let sip_manager = setup().await;
    let transaction = sip_manager.transaction.clone();

    as_downcasted!(
        sip_manager,
        core,
        transaction,
        transport,
        CoreSnitch,
        Transaction,
        TransportSnitch
    );

    assert_eq!(transport.messages.len().await, 0);

    let request: rsip::Request = requests::invite_request();
    let result = transaction
        .new_uas_invite_transaction(
            RequestMsg {
                sip_request: request.clone(),
                ..Randomized::default()
            },
            Some(request.provisional_of(180)),
        )
        .await;
    assert!(result.is_ok());

    assert_eq!(transport.messages.len().await, 1);
    assert_eq!(transaction.inner.uas_state.read().await.len(), 1);

    let result = transaction
        .send(ResponseMsg {
            sip_response: responses::ok_response_from(request.clone()),
            ..Randomized::default()
        })
        .await;
    assert!(result.is_ok());
    assert!(
        transaction
            .is_uas_accepted(request.transaction_id().expect("response transaction id"))
            .await
    );
}

/* ##### completed state ##### */
#[tokio::test]
async fn multiple_invites_on_completed_resends_response() {
    let sip_manager = setup().await;
    let transaction = sip_manager.transaction.clone();

    as_downcasted!(
        sip_manager,
        core,
        transaction,
        transport,
        CoreSnitch,
        Transaction,
        TransportSnitch
    );

    assert_eq!(transport.messages.len().await, 0);

    let request: rsip::Request = requests::invite_request();
    let result = transaction
        .new_uas_invite_transaction(
            RequestMsg {
                sip_request: request.clone(),
                ..Randomized::default()
            },
            Some(request.provisional_of(180)),
        )
        .await;

    let response = responses::redirection_response_from(request.clone());
    let result = transaction
        .send(ResponseMsg {
            sip_response: response.clone(),
            ..Randomized::default()
        })
        .await;

    assert!(
        transaction
            .is_uas_completed(request.transaction_id().expect("response transaction id"))
            .await
    );

    let result = transaction
        .process_incoming_message(
            RequestMsg {
                sip_request: request.clone(),
                ..Randomized::default()
            }
            .into(),
        )
        .await;
    assert_eq!(transport.messages.len().await, 3);
    match transport.messages.last().await {
        TransportMsg {
            sip_message: rsip::SipMessage::Response(resp),
            peer: _,
            transport: _,
        } if resp == response => (),
        _ => panic!("unexpected message state"),
    };
    assert_eq!(transaction.inner.uas_state.read().await.len(), 1);
}

#[tokio::test]
async fn redirect_but_peer_not_responding_with_ack() {
    let sip_manager = setup().await;
    let transaction = sip_manager.transaction.clone();

    as_downcasted!(
        sip_manager,
        core,
        transaction,
        transport,
        CoreSnitch,
        Transaction,
        TransportSnitch
    );

    assert_eq!(transport.messages.len().await, 0);

    let request: rsip::Request = requests::invite_request();
    let result = transaction
        .new_uas_invite_transaction(
            RequestMsg {
                sip_request: request.clone(),
                ..Randomized::default()
            },
            Some(request.provisional_of(180)),
        )
        .await;

    let response = responses::redirection_response_from(request.clone());
    let result = transaction
        .send(ResponseMsg {
            sip_response: response.clone(),
            ..Randomized::default()
        })
        .await;

    assert!(
        transaction
            .is_uas_completed(request.transaction_id().expect("response transaction id"))
            .await
    );
    assert_eq!(transport.messages.len().await, 2);

    advance_for(Duration::from_millis(500)).await;
    assert_eq!(transport.messages.len().await, 3);
    advance_for(Duration::from_millis(1000)).await;
    assert_eq!(transport.messages.len().await, 4);
    advance_for(Duration::from_millis(2000)).await; //3.5
    assert_eq!(transport.messages.len().await, 5);
    advance_for(Duration::from_millis(4000)).await; //7.5
    assert_eq!(transport.messages.len().await, 6);
    advance_for(Duration::from_millis(4000)).await; //11.5
    assert_eq!(transport.messages.len().await, 7);
    advance_for(Duration::from_millis(4000)).await; //15.5
    assert_eq!(transport.messages.len().await, 8);
    advance_for(Duration::from_millis(4000)).await; //19.5
    assert_eq!(transport.messages.len().await, 9);
    advance_for(Duration::from_millis(4000)).await; //23.5
    assert_eq!(transport.messages.len().await, 10);
    advance_for(Duration::from_millis(4000)).await; //27.5
    assert_eq!(transport.messages.len().await, 11);
    advance_for(Duration::from_millis(4000)).await; //31.5
    assert_eq!(transport.messages.len().await, 12);
    //forward time H and check messages + error state
    advance_for(Duration::from_millis(4000)).await; //35.5
    assert_eq!(transport.messages.len().await, 12);

    assert!(
        transaction
            .is_uas_terminated(request.transaction_id().expect("request transaction id"))
            .await
    );
}

#[tokio::test]
async fn with_ack_moves_to_confirmed() {
    use models::RequestExt;

    let sip_manager = setup().await;
    let transaction = sip_manager.transaction.clone();

    as_downcasted!(
        sip_manager,
        core,
        transaction,
        transport,
        CoreSnitch,
        Transaction,
        TransportSnitch
    );

    let request: rsip::Request = requests::invite_request();
    let result = transaction
        .new_uas_invite_transaction(
            RequestMsg {
                sip_request: request.clone(),
                ..Randomized::default()
            },
            Some(request.provisional_of(180)),
        )
        .await
        .expect("new uas invite transaction result");

    let response = ResponseMsg {
        sip_response: responses::redirection_response_from(request.clone()),
        ..Randomized::default()
    };
    let result = transaction
        .send(response.clone())
        .await
        .expect("send transaction result");
    assert!(
        transaction
            .is_uas_completed(request.transaction_id().expect("response transaction id"))
            .await
    );
    assert_eq!(core.messages.len().await, 0);
    assert_eq!(transport.messages.len().await, 2);
    assert_eq!(transaction.inner.uas_state.read().await.len(), 1);

    let result = transaction
        .process_incoming_message(
            RequestMsg {
                sip_request: request.ack_request_with(response.sip_response),
                ..Randomized::default()
            }
            .into(),
        )
        .await;
    assert_eq!(core.messages.len().await, 0);
    assert_eq!(transport.messages.len().await, 2);
    assert_eq!(transaction.inner.uas_state.read().await.len(), 1);
    assert!(
        transaction
            .is_uas_confirmed(request.transaction_id().expect("response transaction id"))
            .await
    );
}

/* ##### accepted state ##### */
#[tokio::test]
async fn multiple_invites_on_accepted_resends_response() {
    let sip_manager = setup().await;
    let transaction = sip_manager.transaction.clone();

    as_downcasted!(
        sip_manager,
        core,
        transaction,
        transport,
        CoreSnitch,
        Transaction,
        TransportSnitch
    );

    assert_eq!(transport.messages.len().await, 0);

    let request: rsip::Request = requests::invite_request();
    let result = transaction
        .new_uas_invite_transaction(
            RequestMsg {
                sip_request: request.clone(),
                ..Randomized::default()
            },
            Some(request.provisional_of(180)),
        )
        .await;

    let response = responses::ok_response_from(request.clone());
    let result = transaction
        .send(ResponseMsg {
            sip_response: response.clone(),
            ..Randomized::default()
        })
        .await;

    assert!(
        transaction
            .is_uas_accepted(request.transaction_id().expect("response transaction id"))
            .await
    );

    let result = transaction
        .process_incoming_message(
            RequestMsg {
                sip_request: request.clone(),
                ..Randomized::default()
            }
            .into(),
        )
        .await;
    assert_eq!(transport.messages.len().await, 3);
    match transport.messages.last().await {
        TransportMsg {
            sip_message: rsip::SipMessage::Response(resp),
            peer: _,
            transport: _,
        } if resp == response => (),
        _ => panic!("unexpected message state"),
    };
    assert_eq!(transaction.inner.uas_state.read().await.len(), 1);
}

#[tokio::test]
async fn ok_but_peer_not_responding_with_ack() {
    let sip_manager = setup().await;
    let transaction = sip_manager.transaction.clone();

    as_downcasted!(
        sip_manager,
        core,
        transaction,
        transport,
        CoreSnitch,
        Transaction,
        TransportSnitch
    );

    assert_eq!(transport.messages.len().await, 0);

    let request: rsip::Request = requests::invite_request();
    let result = transaction
        .new_uas_invite_transaction(
            RequestMsg {
                sip_request: request.clone(),
                ..Randomized::default()
            },
            Some(request.provisional_of(180)),
        )
        .await;

    let response = responses::ok_response_from(request.clone());
    let result = transaction
        .send(ResponseMsg {
            sip_response: response.clone(),
            ..Randomized::default()
        })
        .await;

    assert!(
        transaction
            .is_uas_accepted(request.transaction_id().expect("response transaction id"))
            .await
    );
    assert_eq!(transport.messages.len().await, 2);

    advance_for(Duration::from_millis(1000)).await;
    assert_eq!(transport.messages.len().await, 2);

    assert!(
        transaction
            .is_uas_accepted(request.transaction_id().expect("request transaction id"))
            .await
    );

    advance_for(Duration::from_millis(62 * 1000)).await;
    assert_eq!(transport.messages.len().await, 2);

    assert!(
        transaction
            .is_uas_terminated(request.transaction_id().expect("request transaction id"))
            .await
    );
}

#[tokio::test]
async fn with_multiple_ok_on_accepted() {
    let sip_manager = setup().await;
    let transaction = sip_manager.transaction.clone();

    as_downcasted!(
        sip_manager,
        core,
        transaction,
        transport,
        CoreSnitch,
        Transaction,
        TransportSnitch
    );

    assert_eq!(transport.messages.len().await, 0);

    let request: rsip::Request = requests::invite_request();
    let result = transaction
        .new_uas_invite_transaction(
            RequestMsg {
                sip_request: request.clone(),
                ..Randomized::default()
            },
            Some(request.provisional_of(180)),
        )
        .await
        .expect("new uas invite transaction result");

    assert_eq!(transport.messages.len().await, 1);
    assert_eq!(transaction.inner.uas_state.read().await.len(), 1);

    let result = transaction
        .send(ResponseMsg {
            sip_response: responses::ok_response_from(request.clone()),
            ..Randomized::default()
        })
        .await
        .expect("send transaction result");
    assert!(
        transaction
            .is_uas_accepted(request.transaction_id().expect("response transaction id"))
            .await
    );

    let result = transaction
        .send(ResponseMsg {
            sip_response: responses::ok_response_from(request.clone()),
            ..Randomized::default()
        })
        .await
        .expect("send transaction result");

    assert_eq!(transport.messages.len().await, 3);
    assert_eq!(transaction.inner.uas_state.read().await.len(), 1);
}

#[tokio::test]
async fn with_error_on_second_ok_on_accepted() {
    let sip_manager = setup_with_error_transport().await;
    let transaction = sip_manager.transaction.clone();

    as_downcasted!(
        sip_manager,
        core,
        transaction,
        transport,
        CoreSnitch,
        Transaction,
        TransportErrorSnitch
    );

    transport.turn_fail_switch_off().await;

    let request: rsip::Request = requests::invite_request();
    let result = transaction
        .new_uas_invite_transaction(
            RequestMsg {
                sip_request: request.clone(),
                ..Randomized::default()
            },
            Some(request.provisional_of(180)),
        )
        .await
        .expect("new uas invite transaction result");

    assert_eq!(transport.messages.len().await, 1);
    assert_eq!(transaction.inner.uas_state.read().await.len(), 1);

    let result = transaction
        .send(ResponseMsg {
            sip_response: responses::ok_response_from(request.clone()),
            ..Randomized::default()
        })
        .await
        .expect("send transaction result");
    assert!(
        transaction
            .is_uas_accepted(request.transaction_id().expect("response transaction id"))
            .await
    );

    let result = transaction
        .send(ResponseMsg {
            sip_response: responses::ok_response_from(request.clone()),
            ..Randomized::default()
        })
        .await
        .expect("send transaction result");

    assert_eq!(transport.messages.len().await, 3);
    assert_eq!(transaction.inner.uas_state.read().await.len(), 1);
    assert!(
        transaction
            .is_uas_accepted(request.transaction_id().expect("response transaction id"))
            .await
    );
    transport.turn_fail_switch_on().await;

    let result = transaction
        .send(ResponseMsg {
            sip_response: responses::ok_response_from(request.clone()),
            ..Randomized::default()
        })
        .await;

    assert_eq!(transport.messages.len().await, 3);
    assert_eq!(transaction.inner.uas_state.read().await.len(), 1);

    assert!(
        transaction
            .is_uas_errored(request.transaction_id().expect("response transaction id"))
            .await
    );
    assert!(result.is_err());
}

#[tokio::test]
async fn multiple_ack_received_are_forwarded_to_tu() {
    use models::RequestExt;

    let sip_manager = setup().await;
    let transaction = sip_manager.transaction.clone();

    as_downcasted!(
        sip_manager,
        core,
        transaction,
        transport,
        CoreSnitch,
        Transaction,
        TransportSnitch
    );

    let request: rsip::Request = requests::invite_request();
    let result = transaction
        .new_uas_invite_transaction(
            RequestMsg {
                sip_request: request.clone(),
                ..Randomized::default()
            },
            Some(request.provisional_of(180)),
        )
        .await
        .expect("new uas invite transaction result");

    let response = ResponseMsg {
        sip_response: responses::ok_response_from(request.clone()),
        ..Randomized::default()
    };
    let result = transaction
        .send(response.clone())
        .await
        .expect("send transaction result");
    assert!(
        transaction
            .is_uas_accepted(request.transaction_id().expect("response transaction id"))
            .await
    );
    assert_eq!(core.messages.len().await, 0);
    assert_eq!(transport.messages.len().await, 2);
    assert_eq!(transaction.inner.uas_state.read().await.len(), 1);

    let result = transaction
        .process_incoming_message(
            RequestMsg {
                sip_request: request.ack_request_with(response.sip_response),
                ..Randomized::default()
            }
            .into(),
        )
        .await;
    assert_eq!(core.messages.len().await, 1);
    assert_eq!(transport.messages.len().await, 2);
    assert_eq!(transaction.inner.uas_state.read().await.len(), 1);
}

/* ##### confirmed state ##### */
#[tokio::test]
async fn when_confirmed_acks_have_no_effect() {
    use models::RequestExt;

    let sip_manager = setup().await;
    let transaction = sip_manager.transaction.clone();

    as_downcasted!(
        sip_manager,
        core,
        transaction,
        transport,
        CoreSnitch,
        Transaction,
        TransportSnitch
    );

    let request: rsip::Request = requests::invite_request();
    let result = transaction
        .new_uas_invite_transaction(
            RequestMsg {
                sip_request: request.clone(),
                ..Randomized::default()
            },
            Some(request.provisional_of(180)),
        )
        .await
        .expect("new uas invite transaction result");

    let response = ResponseMsg {
        sip_response: responses::redirection_response_from(request.clone()),
        ..Randomized::default()
    };
    let result = transaction
        .send(response.clone())
        .await
        .expect("send transaction result");

    let result = transaction
        .process_incoming_message(
            RequestMsg {
                sip_request: request.ack_request_with(response.sip_response.clone()),
                ..Randomized::default()
            }
            .into(),
        )
        .await;

    assert_eq!(core.messages.len().await, 0);
    assert_eq!(transport.messages.len().await, 2);
    assert_eq!(transaction.inner.uas_state.read().await.len(), 1);
    assert!(
        transaction
            .is_uas_confirmed(request.transaction_id().expect("response transaction id"))
            .await
    );

    let result = transaction
        .process_incoming_message(
            RequestMsg {
                sip_request: request.ack_request_with(response.sip_response.clone()),
                ..Randomized::default()
            }
            .into(),
        )
        .await;
    let result = transaction
        .process_incoming_message(
            RequestMsg {
                sip_request: request.ack_request_with(response.sip_response),
                ..Randomized::default()
            }
            .into(),
        )
        .await;

    assert_eq!(core.messages.len().await, 0);
    assert_eq!(transport.messages.len().await, 2);
    assert_eq!(transaction.inner.uas_state.read().await.len(), 1);
    assert!(
        transaction
            .is_uas_confirmed(request.transaction_id().expect("response transaction id"))
            .await
    );
}

#[tokio::test]
async fn when_confirmed_when_time_i_kicks_in_move_to_terminated() {
    use models::RequestExt;

    let sip_manager = setup().await;
    let transaction = sip_manager.transaction.clone();

    as_downcasted!(
        sip_manager,
        core,
        transaction,
        transport,
        CoreSnitch,
        Transaction,
        TransportSnitch
    );

    let request: rsip::Request = requests::invite_request();
    let result = transaction
        .new_uas_invite_transaction(
            RequestMsg {
                sip_request: request.clone(),
                ..Randomized::default()
            },
            Some(request.provisional_of(180)),
        )
        .await
        .expect("new uas invite transaction result");

    let response = ResponseMsg {
        sip_response: responses::redirection_response_from(request.clone()),
        ..Randomized::default()
    };
    let result = transaction
        .send(response.clone())
        .await
        .expect("send transaction result");

    let result = transaction
        .process_incoming_message(
            RequestMsg {
                sip_request: request.ack_request_with(response.sip_response.clone()),
                ..Randomized::default()
            }
            .into(),
        )
        .await;

    assert_eq!(core.messages.len().await, 0);
    assert_eq!(transport.messages.len().await, 2);
    assert_eq!(transaction.inner.uas_state.read().await.len(), 1);
    assert!(
        transaction
            .is_uas_confirmed(request.transaction_id().expect("response transaction id"))
            .await
    );

    advance_for(Duration::from_millis(5000)).await;

    assert_eq!(core.messages.len().await, 0);
    assert_eq!(transport.messages.len().await, 2);
    assert_eq!(transaction.inner.uas_state.read().await.len(), 1);
    assert!(
        transaction
            .is_uas_terminated(request.transaction_id().expect("response transaction id"))
            .await
    );
}
