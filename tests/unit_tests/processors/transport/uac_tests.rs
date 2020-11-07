use super::{setup, Setup};
use crate::common::{delay_for, factories::prelude::*};
use common::futures_util::stream::StreamExt;
use common::log::Level;
use models::transport::TransportMsg;
use std::convert::{TryFrom, TryInto};
use std::net::{IpAddr, Ipv4Addr};

#[tokio::test]
async fn incoming_response_asserts_with_wrong_sent_by() {
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

    processor
        .handle_server_message(server_msg.try_into().expect("server to transport msg"))
        .await;

    delay_for(10).await;
    assert!(transport_to_core_stream.try_recv().is_err());
    assert!(transport_to_transaction_stream.try_recv().is_err());

    testing_logger::validate(|captured_logs| {
        assert_eq!(captured_logs.len(), 1);
        assert!(captured_logs[0].body.contains("sent-by address"));
        assert_eq!(captured_logs[0].level, Level::Error);
    });
}

#[tokio::test]
async fn incoming_response_asserts_with_correct_sent_by() {
    use rsip::{
        common::uri::{self, Uri},
        message::HeadersExt,
    };

    let Setup {
        mut processor,
        mut transport_to_core_stream,
        mut transport_to_transaction_stream,
        mut transport_to_server_stream,
    } = setup();

    let response: rsip::Response = responses::response(
        Some(Uri::localhost_with_port(5060)),
        Some(Uri::localhost_with_port(5090)),
    );
    let server_msg = models::server::UdpTuple {
        bytes: response.into(),
        peer: SocketAddrBuilder::localhost_with_port(5090).into(),
    };

    processor
        .handle_server_message(server_msg.try_into().expect("server to transport msg"))
        .await;

    let transport_msg = transport_to_core_stream
        .next()
        .await
        .expect("transport msg");

    delay_for(10).await;
    assert!(transport_to_transaction_stream.try_recv().is_err());
}

#[tokio::test]
async fn outgoing_transaction_request_applies_maddr() {
    use rsip::{
        common::uri::{self, Uri},
        message::HeadersExt,
    };

    let Setup {
        processor,
        mut transport_to_core_stream,
        mut transport_to_transaction_stream,
        mut transport_to_server_stream,
    } = setup();

    let transport_msg = models::transport::TransportMsg {
        peer: SocketAddrBuilder {
            ip_addr: IpAddrBuilder {
                multicast: true,
                ..Default::default()
            }
            .build(),
            ..Default::default()
        }
        .into(),
        ..Randomized::default()
    };

    processor
        .handle_transaction_message(transport_msg.clone())
        .await;

    let udp_tuple = transport_to_server_stream.next().await.expect("udp tuple");
    let request: rsip::Request = udp_tuple
        .bytes
        .try_into()
        .expect("converting bytes to request");
    let via_uri = &request.via_header().expect("via header").uri;

    let maddr_param = via_uri
        .params
        .iter()
        .find(|s| match s {
            uri::Param::Other(key, _) if key == "maddr" => true,
            _ => false,
        })
        .expect("maddr param is missing when address is multicast");
    assert_eq!(
        maddr_param,
        &uri::Param::Other("maddr".into(), Some(transport_msg.peer.ip().to_string()))
    );

    delay_for(10).await;
    assert!(transport_to_core_stream.try_recv().is_err());
    assert!(transport_to_transaction_stream.try_recv().is_err());
}

#[tokio::test]
async fn outgoing_transaction_request_applies_ttl() {
    use rsip::{
        common::uri::{self, Uri},
        message::HeadersExt,
    };

    let Setup {
        processor,
        mut transport_to_core_stream,
        mut transport_to_transaction_stream,
        mut transport_to_server_stream,
    } = setup();

    let transport_msg = TransportMsg {
        peer: SocketAddrBuilder {
            ip_addr: IpAddrBuilder {
                version: IpVersion::V4,
                ..Default::default()
            }
            .build(),
            ..Default::default()
        }
        .into(),
        ..Randomized::default()
    };

    processor
        .handle_transaction_message(transport_msg.clone())
        .await;

    let udp_tuple = transport_to_server_stream.next().await.expect("udp tuple");
    let request: rsip::Request = udp_tuple
        .bytes
        .try_into()
        .expect("converting bytes to request");
    let via_uri = &request.via_header().expect("via header").uri;

    let maddr_param = via_uri
        .params
        .iter()
        .find(|s| match s {
            uri::Param::Other(key, _) if key == "ttl" => true,
            _ => false,
        })
        .expect("ttl param is missing");
    assert_eq!(
        maddr_param,
        &uri::Param::Other("ttl".into(), Some("1".into()))
    );

    delay_for(10).await;
    assert!(transport_to_core_stream.try_recv().is_err());
    assert!(transport_to_transaction_stream.try_recv().is_err());
}

#[tokio::test]
async fn outgoing_transaction_request_applies_sent_by() {
    use rsip::{
        common::uri::{self, Uri},
        message::HeadersExt,
    };

    let Setup {
        processor,
        mut transport_to_core_stream,
        mut transport_to_transaction_stream,
        mut transport_to_server_stream,
    } = setup();

    let transport_msg = TransportMsg {
        peer: SocketAddrBuilder {
            ip_addr: IpAddrBuilder {
                version: IpVersion::V4,
                ..Default::default()
            }
            .build(),
            ..Default::default()
        }
        .into(),
        ..Randomized::default()
    };

    processor
        .handle_transaction_message(transport_msg.clone())
        .await;

    let udp_tuple = transport_to_server_stream.next().await.expect("udp tuple");
    let request: rsip::Request = udp_tuple
        .bytes
        .try_into()
        .expect("converting bytes to request");
    let via_uri = &request.via_header().expect("via header").uri;

    //TODO: this should be configurable through env/yaml config
    assert_eq!(via_uri.host_with_port.to_string(), "127.0.0.1:5060");

    delay_for(10).await;
    assert!(transport_to_core_stream.try_recv().is_err());
    assert!(transport_to_transaction_stream.try_recv().is_err());
}

#[tokio::test]
async fn outgoing_core_request_applies_maddr() {
    use rsip::{
        common::uri::{self, Uri},
        message::HeadersExt,
    };

    let Setup {
        processor,
        mut transport_to_core_stream,
        mut transport_to_transaction_stream,
        mut transport_to_server_stream,
    } = setup();

    let transport_msg = TransportMsg {
        peer: SocketAddrBuilder {
            ip_addr: IpAddrBuilder {
                multicast: true,
                ..Default::default()
            }
            .build(),
            ..Default::default()
        }
        .into(),
        ..Randomized::default()
    };

    processor.handle_core_message(transport_msg.clone()).await;

    let udp_tuple = transport_to_server_stream.next().await.expect("udp tuple");
    let request: rsip::Request = udp_tuple
        .bytes
        .try_into()
        .expect("converting bytes to request");
    let via_uri = &request.via_header().expect("via header").uri;

    let maddr_param = via_uri
        .params
        .iter()
        .find(|s| match s {
            uri::Param::Other(key, _) if key == "maddr" => true,
            _ => false,
        })
        .expect("maddr param is missing when address is multicast");
    assert_eq!(
        maddr_param,
        &uri::Param::Other("maddr".into(), Some(transport_msg.peer.ip().to_string()))
    );

    delay_for(10).await;
    assert!(transport_to_core_stream.try_recv().is_err());
    assert!(transport_to_transaction_stream.try_recv().is_err());
}

#[tokio::test]
async fn outgoing_core_request_applies_ttl() {
    use rsip::{
        common::uri::{self, Uri},
        message::HeadersExt,
    };

    let Setup {
        processor,
        mut transport_to_core_stream,
        mut transport_to_transaction_stream,
        mut transport_to_server_stream,
    } = setup();

    let transport_msg = TransportMsg {
        peer: SocketAddrBuilder {
            ip_addr: IpAddrBuilder {
                version: IpVersion::V4,
                ..Default::default()
            }
            .build(),
            ..Default::default()
        }
        .into(),
        ..Randomized::default()
    };

    processor.handle_core_message(transport_msg.clone()).await;

    let udp_tuple = transport_to_server_stream.next().await.expect("udp tuple");
    let request: rsip::Request = udp_tuple
        .bytes
        .try_into()
        .expect("converting bytes to request");
    let via_uri = &request.via_header().expect("via header").uri;

    let maddr_param = via_uri
        .params
        .iter()
        .find(|s| match s {
            uri::Param::Other(key, _) if key == "ttl" => true,
            _ => false,
        })
        .expect("ttl param is missing");
    assert_eq!(
        maddr_param,
        &uri::Param::Other("ttl".into(), Some("1".into()))
    );

    delay_for(10).await;
    assert!(transport_to_core_stream.try_recv().is_err());
    assert!(transport_to_transaction_stream.try_recv().is_err());
}

#[tokio::test]
async fn outgoing_core_request_applies_sent_by() {
    use rsip::{
        common::uri::{self, Uri},
        message::HeadersExt,
    };

    let Setup {
        processor,
        mut transport_to_core_stream,
        mut transport_to_transaction_stream,
        mut transport_to_server_stream,
    } = setup();

    let transport_msg = TransportMsg {
        peer: SocketAddrBuilder {
            ip_addr: IpAddrBuilder {
                version: IpVersion::V4,
                ..Default::default()
            }
            .build(),
            ..Default::default()
        }
        .into(),
        ..Randomized::default()
    };

    processor.handle_core_message(transport_msg.clone()).await;

    let udp_tuple = transport_to_server_stream.next().await.expect("udp tuple");
    let request: rsip::Request = udp_tuple
        .bytes
        .try_into()
        .expect("converting bytes to request");
    let via_uri = &request.via_header().expect("via header").uri;

    //TODO: this should be configurable through env/yaml config
    assert_eq!(via_uri.host_with_port.to_string(), "127.0.0.1:5060");

    delay_for(10).await;
    assert!(transport_to_core_stream.try_recv().is_err());
    assert!(transport_to_transaction_stream.try_recv().is_err());
}
