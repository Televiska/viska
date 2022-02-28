use crate::Error;
use common::rsip::{self, prelude::*};
use std::net::SocketAddr;

//outgoing
pub fn apply_request_defaults(
    mut request: rsip::Request,
    peer: SocketAddr,
    _transport: rsip::Transport,
) -> Result<rsip::Request, Error> {
    apply_via_maddr_address(
        request.via_header_mut().expect("via header is missing!"),
        &peer,
    )?;
    apply_via_ttl(
        request.via_header_mut().expect("via header is missing!"),
        &peer,
    )?;
    apply_via_sent_by(request.via_header_mut().expect("via header is missing!"))?;

    Ok(request)
}

//incoming
pub fn apply_response_defaults(
    response: rsip::Response,
    _peer: SocketAddr,
    _transport: rsip::Transport,
) -> Result<rsip::Response, Error> {
    assert_sent_by_value(response.via_header().expect("via header missing"))?;
    Ok(response)
}

pub fn apply_via_maddr_address(
    via_header: &mut rsip::headers::Via,
    peer: &SocketAddr,
) -> Result<(), Error> {
    use rsip::{param::Maddr, Param};

    if peer.ip().is_multicast() {
        via_header.replace(
            via_header
                .typed()?
                .with_param(Param::Maddr(Maddr::new(peer.ip().to_string()))),
        );
    }

    Ok(())
}

pub fn apply_via_ttl(via_header: &mut rsip::headers::Via, peer: &SocketAddr) -> Result<(), Error> {
    use rsip::{param::Ttl, Param};

    if peer.ip().is_ipv4() {
        via_header.replace(via_header.typed()?.with_param(Param::Ttl(Ttl::new("1"))));
    }

    Ok(())
}

pub fn apply_via_sent_by(via_header: &mut rsip::headers::Via) -> Result<(), Error> {
    let typed_via_header = via_header.typed()?;

    let mut uri = typed_via_header.uri.clone();
    uri.host_with_port = common::CONFIG.default_addr();
    via_header.replace(typed_via_header.to_string());

    Ok(())
}

pub fn assert_sent_by_value(via_header: &rsip::headers::Via) -> Result<(), Error> {
    let typed_via_header = via_header.typed()?;

    if common::CONFIG.contains_addr(&typed_via_header.uri.host_with_port) {
        Ok(())
    } else {
        Err(Error::custom(format!(
            "sent-by address ({:?}) is different from listen address",
            typed_via_header.uri.host_with_port,
        )))
    }
}
