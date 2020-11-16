use crate::common::{delay_for, factories::prelude::*};
use common::futures_util::stream::StreamExt;
use common::log::Level;
use models::transport::TransportMsg;
use sip_server::transport::processor::Processor as TransportProcessor;
use std::convert::{TryFrom, TryInto};
use std::net::{IpAddr, Ipv4Addr};

#[tokio::test]
async fn incoming_request_with_other_sent_by_adds_received_param() {
    use rsip::{
        common::uri::{self, Uri},
        message::HeadersExt,
    };

    let processor = TransportProcessor::default();

    let mut request: rsip::Request = requests::request(
        Some(Uri::localhost_with_port(5060)),
        Some(Uri::localhost_with_port(5090)),
    );
    let server_msg = models::server::UdpTuple {
        bytes: request.into(),
        peer: (IpAddr::V4(Ipv4Addr::new(196, 168, 0, 1)), 5061).into(),
    };

    let message = processor
        .process_incoming_message(server_msg.try_into().expect("server to transport msg"))
        .await
        .expect("processor processing failed");
    let request: rsip::Request = message
        .sip_message
        .try_into()
        .expect("transport msg to request");
    let via_uri = &request.via_header().expect("via header").uri;
    let received_param = via_uri
        .params
        .iter()
        .find(|s| match s {
            uri::Param::Received(domain) => true,
            _ => false,
        })
        .expect("received param is missing when via address is different from peer");
}

#[tokio::test]
async fn incoming_request_with_same_sent_by_param() {
    use rsip::{
        common::uri::{self, Uri},
        message::HeadersExt,
    };

    let processor = TransportProcessor::default();

    let mut request: rsip::Request = requests::request(
        Some(Uri::localhost_with_port(5060)),
        Some(Uri::localhost_with_port(5090)),
    );
    let server_msg = models::server::UdpTuple {
        bytes: request.into(),
        peer: (IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5060).into(),
    };

    let message = processor
        .process_incoming_message(server_msg.try_into().expect("server to transport msg"))
        .await
        .expect("processor processing failed");
    let request: rsip::Request = message
        .sip_message
        .try_into()
        .expect("transport msg to request");
    let via_uri = &request.via_header().expect("via header").uri;
    let received_param = via_uri.params.iter().find(|s| match s {
        uri::Param::Received(domain) => true,
        _ => false,
    });
    assert_eq!(received_param, None);
}
