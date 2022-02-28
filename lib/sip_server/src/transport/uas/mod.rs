use crate::Error;
use common::rsip::{self, prelude::*};
use std::net::SocketAddr;

//incoming
pub fn apply_request_defaults(
    mut request: rsip::Request,
    peer: SocketAddr,
    _transport: rsip::Transport,
) -> Result<rsip::Request, Error> {
    use super::uas::*;

    apply_received_value(request.via_header_mut().expect("via header missing"), &peer)?;
    Ok(request)
}

//outgoing
pub fn apply_response_defaults(
    response: rsip::Response,
    _peer: SocketAddr,
    _transport: rsip::Transport,
) -> rsip::Response {
    response
}

pub fn apply_received_value(
    via_header: &mut rsip::headers::Via,
    peer: &SocketAddr,
) -> Result<(), Error> {
    use rsip::{param::Received, Host, HostWithPort, Param};

    let typed_via_header = via_header.typed()?;

    match (typed_via_header.uri.host_with_port.clone(), peer) {
        (
            HostWithPort {
                host: Host::Domain(_),
                ..
            },
            _,
        ) => via_header.replace(
            typed_via_header.with_param(Param::Received(Received::new(peer.clone().to_string()))),
        ),
        (
            HostWithPort {
                host: Host::IpAddr(listen_addr),
                port: Some(port),
            },
            _,
        ) if (listen_addr != peer.ip()) || (*port.value() != peer.port()) => via_header.replace(
            typed_via_header.with_param(Param::Received(Received::new(peer.clone().to_string()))),
        ),
        (
            HostWithPort {
                host: Host::IpAddr(listen_addr),
                port: None,
            },
            _,
        ) if listen_addr != peer.ip() => via_header.replace(
            typed_via_header.with_param(Param::Received(Received::new(peer.clone().to_string()))),
        ),
        (_, _) => (),
    }

    Ok(())
}
