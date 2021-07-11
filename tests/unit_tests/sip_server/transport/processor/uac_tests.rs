use crate::common::{delay_for, factories::prelude::*};
use common::futures_util::stream::StreamExt;
use common::log::Level;
use common::rsip::prelude::*;
use models::transport::TransportMsg;
use sip_server::transport::processor::Processor as TransportProcessor;
use std::any::Any;
use std::convert::{TryFrom, TryInto};
use std::net::{IpAddr, Ipv4Addr};

#[tokio::test]
async fn incoming_response_asserts_with_wrong_sent_by() -> Result<(), sip_server::Error> {
    use rsip::common::Uri;

    let processor = TransportProcessor::default();

    let mut response: rsip::Response =
        responses::response(Some(Uri::default()), Some(Uri::default().with_port(5090)));
    let mut via_header = response.via_header_mut()?;
    via_header.replace(
        via_header
            .typed()?
            .with_uri(Uri::default().with_port(5070).into()),
    );

    let server_msg = models::server::UdpTuple {
        bytes: response.into(),
        peer: SocketAddrBuilder::localhost_with_port(5090).into(),
    };

    match processor
        .process_incoming_message(server_msg.try_into()?)
        .await
    {
        Err(sip_server::Error {
            kind: sip_server::ErrorKind::Custom(error),
        }) => assert!(error.contains("sent-by") && error.contains("different")),
        res => panic!("unexpected result: {:?}", res),
    }

    Ok(())
}

#[tokio::test]
async fn incoming_response_asserts_with_correct_sent_by() -> Result<(), sip_server::Error> {
    use rsip::common::Uri;

    let processor = TransportProcessor::default();

    let response: rsip::Response =
        responses::response(Some(Uri::default()), Some(Uri::default().with_port(5090)));
    let server_msg = models::server::UdpTuple {
        bytes: response.into(),
        peer: SocketAddrBuilder::localhost_with_port(5090).into(),
    };

    assert!(processor
        .process_incoming_message(server_msg.try_into()?)
        .await
        .is_ok());

    Ok(())
}

#[tokio::test]
async fn outgoing_transaction_request_applies_maddr() -> Result<(), sip_server::Error> {
    use rsip::common::uri::{Maddr, Param, Uri};

    let processor = TransportProcessor::default();

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

    let message = processor.process_outgoing_message(transport_msg.clone())?;

    let request: rsip::Request = message.sip_message.try_into()?;
    let typed_via_uri = &request.via_header()?.typed()?;

    let maddr_param = typed_via_uri
        .params
        .iter()
        .find(|s| matches!(s, Param::Maddr(_)))
        .expect("no maddr found");

    assert_eq!(
        maddr_param,
        &Param::Maddr(Maddr::new(transport_msg.peer.ip().to_string()))
    );

    Ok(())
}

#[tokio::test]
async fn outgoing_transaction_request_applies_ttl() -> Result<(), sip_server::Error> {
    use rsip::common::uri::{Param, Ttl, Uri};

    let processor = TransportProcessor::default();

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

    let message = processor.process_outgoing_message(transport_msg.clone())?;

    let request: rsip::Request = message.sip_message.try_into()?;
    let typed_via_header = &request.via_header()?.typed()?;

    let maddr_param = typed_via_header
        .params
        .iter()
        .find(|s| matches!(s, Param::Ttl(_)))
        .expect("ttl param is missing");

    assert_eq!(maddr_param, &Param::Ttl(Ttl::new("1")));

    Ok(())
}

#[tokio::test]
async fn outgoing_transaction_request_applies_sent_by() -> Result<(), sip_server::Error> {
    use rsip::common::uri::Uri;

    let processor = TransportProcessor::default();

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

    let message = processor.process_outgoing_message(transport_msg.clone())?;
    let request: rsip::Request = message.sip_message.try_into()?;
    let typed_via_header = &request.via_header()?.typed()?;

    assert_eq!(
        typed_via_header.uri.host_with_port,
        common::CONFIG.default_addr()
    );

    Ok(())
}

#[tokio::test]
async fn outgoing_core_request_applies_maddr() -> Result<(), sip_server::Error> {
    use rsip::common::uri::{Maddr, Param, Uri};

    let processor = TransportProcessor::default();

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

    let message = processor.process_outgoing_message(transport_msg.clone())?;
    let request: rsip::Request = message.sip_message.try_into()?;
    let typed_via_header = &request.via_header()?.typed()?;

    let maddr_param = typed_via_header
        .params
        .iter()
        .find(|s| matches!(s, Param::Maddr(_)))
        .expect("maddr param is missing when address is multicast");

    assert_eq!(
        maddr_param,
        &Param::Maddr(Maddr::new(transport_msg.peer.ip().to_string()))
    );

    Ok(())
}

#[tokio::test]
async fn outgoing_core_request_applies_ttl() -> Result<(), sip_server::Error> {
    use rsip::common::uri::{Param, Ttl, Uri};

    let processor = TransportProcessor::default();

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

    let message = processor.process_outgoing_message(transport_msg.clone())?;
    let request: rsip::Request = message.sip_message.try_into()?;
    let typed_via_header = &request.via_header()?.typed()?;

    let maddr_param = typed_via_header
        .params
        .iter()
        .find(|s| matches!(s, Param::Ttl(_)))
        .expect("ttl param is missing");

    assert_eq!(maddr_param, &Param::Ttl(Ttl::new("1")));

    Ok(())
}

#[tokio::test]
async fn outgoing_core_request_applies_sent_by() -> Result<(), sip_server::Error> {
    use rsip::common::uri::Uri;

    let processor = TransportProcessor::default();

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

    let message = processor.process_outgoing_message(transport_msg.clone())?;
    let request: rsip::Request = message.sip_message.try_into()?;
    let typed_via_header = &request.via_header()?.typed()?;

    assert_eq!(
        typed_via_header.uri.host_with_port,
        common::CONFIG.default_addr()
    );

    Ok(())
}
