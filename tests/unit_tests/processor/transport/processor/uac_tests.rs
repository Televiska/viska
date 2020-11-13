use crate::common::{delay_for, factories::prelude::*};
use common::futures_util::stream::StreamExt;
use common::log::Level;
use models::transport::TransportMsg;
use std::any::Any;
use std::convert::{TryFrom, TryInto};
use std::net::{IpAddr, Ipv4Addr};

#[tokio::test]
async fn incoming_response_asserts_with_wrong_sent_by() {
    use rsip::{
        common::uri::{self, Uri},
        message::HeadersExt,
    };

    let processor = processor::transport::processor::Processor::default();

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

    match processor
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

    let processor = processor::transport::processor::Processor::default();

    let response: rsip::Response = responses::response(
        Some(Uri::localhost_with_port(5060)),
        Some(Uri::localhost_with_port(5090)),
    );
    let server_msg = models::server::UdpTuple {
        bytes: response.into(),
        peer: SocketAddrBuilder::localhost_with_port(5090).into(),
    };

    assert!(processor
        .process_incoming_message(server_msg.try_into().expect("server to transport msg"))
        .await
        .is_ok());
}

#[tokio::test]
async fn outgoing_transaction_request_applies_maddr() {
    use rsip::{
        common::uri::{self, Uri},
        message::HeadersExt,
    };

    let processor = processor::transport::processor::Processor::default();

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

    let message = processor.process_outgoing_message(transport_msg.clone());

    let request: rsip::Request = message
        .sip_message
        .try_into()
        .expect("transport msg to request");
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
}

#[tokio::test]
async fn outgoing_transaction_request_applies_ttl() {
    use rsip::{
        common::uri::{self, Uri},
        message::HeadersExt,
    };

    let processor = processor::transport::processor::Processor::default();

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

    let message = processor.process_outgoing_message(transport_msg.clone());

    let request: rsip::Request = message
        .sip_message
        .try_into()
        .expect("transport msg to request");
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
}

#[tokio::test]
async fn outgoing_transaction_request_applies_sent_by() {
    use rsip::{
        common::uri::{self, Uri},
        message::HeadersExt,
    };

    let processor = processor::transport::processor::Processor::default();

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

    let message = processor.process_outgoing_message(transport_msg.clone());
    let request: rsip::Request = message
        .sip_message
        .try_into()
        .expect("transport msg to request");
    let via_uri = &request.via_header().expect("via header").uri;

    //TODO: this should be configurable through env/yaml config
    assert_eq!(via_uri.host_with_port.to_string(), "127.0.0.1:5060");
}

#[tokio::test]
async fn outgoing_core_request_applies_maddr() {
    use rsip::{
        common::uri::{self, Uri},
        message::HeadersExt,
    };

    let processor = processor::transport::processor::Processor::default();

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

    let message = processor.process_outgoing_message(transport_msg.clone());
    let request: rsip::Request = message
        .sip_message
        .try_into()
        .expect("transport msg to request");
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
}

#[tokio::test]
async fn outgoing_core_request_applies_ttl() {
    use rsip::{
        common::uri::{self, Uri},
        message::HeadersExt,
    };

    let processor = processor::transport::processor::Processor::default();

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

    let message = processor.process_outgoing_message(transport_msg.clone());
    let request: rsip::Request = message
        .sip_message
        .try_into()
        .expect("transport msg to request");
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
}

#[tokio::test]
async fn outgoing_core_request_applies_sent_by() {
    use rsip::{
        common::uri::{self, Uri},
        message::HeadersExt,
    };

    let processor = processor::transport::processor::Processor::default();

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

    let message = processor.process_outgoing_message(transport_msg.clone());
    let request: rsip::Request = message
        .sip_message
        .try_into()
        .expect("transport msg to request");
    let via_uri = &request.via_header().expect("via header").uri;

    //TODO: this should be configurable through env/yaml config
    assert_eq!(via_uri.host_with_port.to_string(), "127.0.0.1:5060");
}
