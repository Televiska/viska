use crate::Error;
use rsip::{
    common::{uri::HostWithPort, Transport},
    headers::Via,
    message::HeadersExt,
    Request, Response, SipMessage,
};
use std::net::SocketAddr;

//incoming
pub fn apply_request_defaults(
    mut request: Request,
    peer: SocketAddr,
    _transport: Transport,
) -> Result<SipMessage, Error> {
    use super::uas::*;

    apply_received_value(request.via_header_mut().expect("via header missing"), &peer)?;
    Ok(request.into())
}

//outgoing
pub fn apply_response_defaults(
    response: Response,
    _peer: SocketAddr,
    _transport: Transport,
) -> SipMessage {
    response.into()
}

pub fn apply_received_value(via_header: &mut Via, peer: &SocketAddr) -> Result<(), Error> {
    use rsip::common::uri::Param;

    match (via_header.uri.host_with_port.clone(), peer) {
        (HostWithPort::Domain(_), _) => {
            let mut uri = via_header.uri.clone();
            uri.params.push(Param::Received(peer.clone().into()));
            via_header.uri = uri;
        }
        (HostWithPort::SocketAddr(listen_addr), SocketAddr::V4(_))
            if (listen_addr.ip() != peer.ip()) || (listen_addr.port() != peer.port()) =>
        {
            let mut uri = via_header.uri.clone();
            uri.params.push(Param::Received(peer.clone().into()));
            via_header.uri = uri;
        }
        (HostWithPort::IpAddr(_), _) => {
            let mut uri = via_header.uri.clone();
            uri.params.push(Param::Received(peer.clone().into()));
            via_header.uri = uri;
        }
        (_, _) => (),
    }

    Ok(())
}
