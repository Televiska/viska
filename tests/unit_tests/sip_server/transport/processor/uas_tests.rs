use crate::common::factories::prelude::*;
use common::rsip::{self, prelude::*};
use sip_server::transport::processor::Processor as TransportProcessor;
use std::convert::TryInto;
use std::net::{IpAddr, Ipv4Addr};

#[tokio::test]
async fn incoming_request_with_other_sent_by_adds_received_param() -> Result<(), sip_server::Error>
{
    use rsip::{Param, Uri};

    let processor = TransportProcessor::default();

    let request: rsip::Request =
        requests::request(Some(Uri::default()), Some(Uri::default().with_port(5090)));
    let server_msg = models::transport::UdpTuple {
        bytes: request.into(),
        peer: (IpAddr::V4(Ipv4Addr::new(196, 168, 0, 1)), 5061).into(),
    };

    let message = processor
        .process_incoming_request(server_msg.try_into()?)
        .await?;
    let request: rsip::Request = message.sip_request;
    let typed_via_header = &request.via_header()?.typed()?;
    let _received_param = typed_via_header
        .params
        .iter()
        .find(|s| matches!(s, Param::Received(_)))
        .expect("received param is missing when via address is different from peer");

    Ok(())
}

#[tokio::test]
async fn incoming_request_with_same_sent_by_param() -> Result<(), sip_server::Error> {
    use rsip::{Param, Uri};

    let processor = TransportProcessor::default();

    let request: rsip::Request =
        requests::request(Some(Uri::default()), Some(Uri::default().with_port(5090)));
    let server_msg = models::transport::UdpTuple {
        bytes: request.into(),
        peer: common::CONFIG.default_addr().try_into()?,
    };

    let message = processor
        .process_incoming_request(server_msg.try_into()?)
        .await?;
    let request: rsip::Request = message.sip_request;
    let typed_via_header = &request.via_header()?.typed()?;
    let received_param = typed_via_header
        .params
        .iter()
        .find(|s| matches!(s, Param::Received(_)));
    assert_eq!(received_param, None);

    Ok(())
}
