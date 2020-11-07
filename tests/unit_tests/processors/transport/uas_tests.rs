use super::{setup, Setup};
use crate::common::{delay_for, factories::prelude::*};
use common::futures_util::stream::StreamExt;
use common::log::Level;
use models::transport::TransportMsg;
use std::convert::{TryFrom, TryInto};
use std::net::{IpAddr, Ipv4Addr};

#[tokio::test]
async fn incoming_request_with_other_sent_by_adds_received_param() {
    use rsip::{
        common::uri::{self, Uri},
        message::HeadersExt,
    };

    testing_logger::setup();

    let Setup {
        mut processor,
        mut transport_to_core_stream,
        mut transport_to_transaction_stream,
        mut transport_to_server_stream,
    } = setup();

    let mut response: rsip::Request = requests::request(
        Some(Uri::localhost_with_port(5060)),
        Some(Uri::localhost_with_port(5090)),
    );
    let server_msg = models::server::UdpTuple {
        bytes: response.into(),
        peer: (IpAddr::V4(Ipv4Addr::new(196, 168, 0, 1)), 5061).into(),
    };

    processor
        .handle_server_message(server_msg.try_into().expect("server to transport msg"))
        .await;

    let transport_msg = transport_to_core_stream
        .next()
        .await
        .expect("transport msg");
    let request: rsip::Request = transport_msg
        .sip_message
        .try_into()
        .expect("to transport msg");
    let via_uri = &request.via_header().expect("via header").uri;
    let received_param = via_uri
        .params
        .iter()
        .find(|s| match s {
            uri::Param::Received(domain) => true,
            _ => false,
        })
        .expect("received param is missing when via address is different from peer");

    delay_for(10).await;
    assert!(transport_to_core_stream.try_recv().is_err());
    assert!(transport_to_transaction_stream.try_recv().is_err());
}

#[tokio::test]
async fn incoming_request_with_same_sent_by_param() {
    use rsip::{
        common::uri::{self, Uri},
        message::HeadersExt,
    };

    testing_logger::setup();

    let Setup {
        mut processor,
        mut transport_to_core_stream,
        mut transport_to_transaction_stream,
        mut transport_to_server_stream,
    } = setup();

    let mut response: rsip::Request = requests::request(
        Some(Uri::localhost_with_port(5060)),
        Some(Uri::localhost_with_port(5090)),
    );
    let server_msg = models::server::UdpTuple {
        bytes: response.into(),
        peer: (IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5060).into(),
    };

    processor
        .handle_server_message(server_msg.try_into().expect("server to transport msg"))
        .await;

    let transport_msg = transport_to_core_stream
        .next()
        .await
        .expect("transport msg");
    let request: rsip::Request = transport_msg
        .sip_message
        .try_into()
        .expect("to transport msg");
    let via_uri = &request.via_header().expect("via header").uri;
    let received_param = via_uri.params.iter().find(|s| match s {
        uri::Param::Received(domain) => true,
        _ => false,
    });
    assert_eq!(received_param, None);

    delay_for(10).await;
    assert!(transport_to_core_stream.try_recv().is_err());
    assert!(transport_to_transaction_stream.try_recv().is_err());
}
