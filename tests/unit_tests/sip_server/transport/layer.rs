/*
#[tokio::test]
async fn incoming_response_asserts_with_wrong_sent_by() {
    use rsip::{
        common::uri::{self, Uri},
        message::HeadersExt,
    };

    let manager = super::setup().await;

    let mut response: rsip::Response = responses::response(
        Some(Uri::localhost_with_port(5060)),
        Some(Uri::localhost_with_port(5090)),
    );
    let via_header = response.via_header_mut().expect("via_header");
    via_header.uri = Uri::localhost_with_port(5070).into();
    let server_msg = models::server::UdpTuple {
        bytes: response.into(),
        peer: SocketAddrBuilder::localhost_with_port(5090).into(),
    };

    match manager
        .transport
        .process_incoming_message(server_msg.try_into().expect("server to transport msg"))
        .await
    {
        Err(processor::Error {
            kind: processor::ErrorKind::Custom(error),
        }) => assert!(error.contains("sent-by") && error.contains("different")),
        _ => panic!("unexpected result"),
    }
}

#[tokio::test]
async fn incoming_response_asserts_with_correct_sent_by() {
    use rsip::{
        common::uri::{self, Uri},
        message::HeadersExt,
    };

    let manager = super::setup().await;

    let response: rsip::Response = responses::response(
        Some(Uri::localhost_with_port(5060)),
        Some(Uri::localhost_with_port(5090)),
    );
    let server_msg = models::server::UdpTuple {
        bytes: response.into(),
        peer: SocketAddrBuilder::localhost_with_port(5090).into(),
    };

    assert!(manager
        .transport
        .process_incoming_message(server_msg.try_into().expect("server to transport msg"))
        .await
        .is_ok());

    let core = manager.core.clone();
    let core = as_any!(core, super::CoreSnitch);

    assert_eq!(core.messages.lock().await.len(), 1);
}*/
