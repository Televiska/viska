use crate::Error;
use rsip::{
    common::{uri::HostWithPort, Transport},
    headers::Via,
    message::HeadersExt,
    Request, Response, SipMessage,
};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

//outgoing
pub fn apply_request_defaults(
    mut request: Request,
    peer: SocketAddr,
    _transport: Transport,
) -> SipMessage {
    apply_via_maddr_address(
        request.via_header_mut().expect("via header is missing!"),
        &peer,
    );
    apply_via_ttl(
        request.via_header_mut().expect("via header is missing!"),
        &peer,
    );
    apply_via_sent_by(request.via_header_mut().expect("via header is missing!"));

    request.into()
}

//incoming
pub fn apply_response_defaults(
    response: Response,
    _peer: SocketAddr,
    _transport: Transport,
) -> Result<SipMessage, Error> {
    assert_sent_by_value(response.via_header().expect("via header missing"))?;
    Ok(response.into())
}

pub fn apply_via_maddr_address(via_header: &mut Via, peer: &SocketAddr) {
    use rsip::common::uri::Param;

    if peer.ip().is_multicast() {
        let mut uri = via_header.uri.clone();
        uri.params
            .push(Param::Other("maddr".into(), Some(peer.ip().to_string())));
        via_header.uri = uri;
    }
}

pub fn apply_via_ttl(via_header: &mut Via, peer: &SocketAddr) {
    use rsip::common::uri::Param;

    if peer.ip().is_ipv4() {
        let mut uri = via_header.uri.clone();
        uri.params
            .push(Param::Other("ttl".into(), Some("1".into())));
        via_header.uri = uri;
    }
}

pub fn apply_via_sent_by(via_header: &mut Via) {
    let mut uri = via_header.uri.clone();
    uri.host_with_port = default_listen_address();
    via_header.uri = uri;
}

pub fn assert_sent_by_value(via_header: &Via) -> Result<(), Error> {
    if via_header.uri.host_with_port == default_listen_address() {
        Ok(())
    } else {
        Err(Error::custom(format!(
            "sent-by address ({:?}) is different from listen address",
            via_header.uri.host_with_port,
        )))
    }
}

//TODO: take domain from config/yaml
//TODO: add a config
fn default_listen_address() -> HostWithPort {
    HostWithPort::SocketAddr(SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        5060,
    ))
}
