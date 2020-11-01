use crate::Error;
use models::{server::UdpTuple, transport::TransportMsg};
use rsip::{
    common::{uri::HostWithPort, Transport},
    headers::Via,
    message::HeadersExt,
    Request, Response, SipMessage,
};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::sync::mpsc::Sender;

//transport processor
#[allow(dead_code)]
pub struct Processor {
    self_to_core_sink: Sender<TransportMsg>,
    self_to_transaction_sink: Sender<TransportMsg>,
    self_to_server_sink: Sender<UdpTuple>,
}

impl Processor {
    pub fn new(
        self_to_core_sink: Sender<TransportMsg>,
        self_to_transaction_sink: Sender<TransportMsg>,
        self_to_server_sink: Sender<UdpTuple>,
    ) -> Self {
        Self {
            self_to_core_sink,
            self_to_transaction_sink,
            self_to_server_sink,
        }
    }

    pub async fn handle_server_message(&self, msg: TransportMsg) {
        //let mut self_to_transaction_sink = self.self_to_transaction_sink.clone();
        let mut self_to_core_sink = self.self_to_core_sink.clone();

        let message = self.process_incoming_message(msg).await;

        match message {
            Ok(message) => {
                if self_to_core_sink.send(message).await.is_err() {
                    common::log::error!("failed to send");
                }
            }
            Err(error) => common::log::error!("failed process incoming message: {:?}", error),
        }
    }

    pub async fn handle_transaction_message(&self, msg: TransportMsg) {
        let mut self_to_server_sink = self.self_to_server_sink.clone();

        let message = self.process_outgoing_message(msg).await;

        if self_to_server_sink.send(message.into()).await.is_err() {
            common::log::error!("failed to send to server from transport processor");
        }
    }

    //TODO: consider merging that with transaction into something like `handle_outgoing_message`
    pub async fn handle_core_message(&self, msg: TransportMsg) {
        let mut self_to_server_sink = self.self_to_server_sink.clone();

        let message = self.process_outgoing_message(msg).await;

        if self_to_server_sink.send(message.into()).await.is_err() {
            common::log::error!("failed to send to server from transport processor");
        }
    }

    async fn process_incoming_message(&self, msg: TransportMsg) -> Result<TransportMsg, Error> {
        let TransportMsg {
            sip_message,
            peer,
            transport,
        } = msg;

        let sip_message = match sip_message {
            SipMessage::Request(request) => {
                self.apply_incoming_request_defaults(request, peer, transport.clone())?
            }
            SipMessage::Response(response) => {
                self.apply_incoming_response_defaults(response, peer, transport.clone())?
            }
        };

        Ok(TransportMsg {
            sip_message,
            peer,
            transport,
        })
    }

    async fn process_outgoing_message(&self, msg: TransportMsg) -> TransportMsg {
        let TransportMsg {
            sip_message,
            peer,
            transport,
        } = msg;

        let sip_message = match sip_message {
            SipMessage::Request(request) => {
                self.apply_outgoing_request_defaults(request, peer, transport.clone())
            }
            SipMessage::Response(response) => {
                self.apply_outgoing_response_defaults(response, peer, transport.clone())
            }
        };

        TransportMsg {
            sip_message,
            peer,
            transport,
        }
    }

    fn apply_outgoing_request_defaults(
        &self,
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

    fn apply_outgoing_response_defaults(
        &self,
        response: Response,
        _peer: SocketAddr,
        _transport: Transport,
    ) -> SipMessage {
        response.into()
    }

    fn apply_incoming_request_defaults(
        &self,
        mut request: Request,
        peer: SocketAddr,
        _transport: Transport,
    ) -> Result<SipMessage, Error> {
        apply_received_value(request.via_header_mut().expect("via header missing"), &peer)?;
        Ok(request.into())
    }

    fn apply_incoming_response_defaults(
        &self,
        response: Response,
        _peer: SocketAddr,
        _transport: Transport,
    ) -> Result<SipMessage, Error> {
        assert_sent_by_value(response.via_header().expect("via header missing"))?;
        Ok(response.into())
    }
}

fn assert_sent_by_value(via_header: &Via) -> Result<(), Error> {
    if via_header.uri.host_with_port == default_listen_address() {
        Ok(())
    } else {
        Err(Error::custom(format!(
            "sent-by address ({:?}) is different from listen address",
            via_header.uri.host_with_port,
        )))
    }
}

fn apply_received_value(via_header: &mut Via, peer: &SocketAddr) -> Result<(), Error> {
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

fn apply_via_maddr_address(via_header: &mut Via, peer: &SocketAddr) {
    use rsip::common::uri::Param;

    if peer.ip().is_multicast() {
        let mut uri = via_header.uri.clone();
        uri.params
            .push(Param::Other("maddr".into(), Some(peer.ip().to_string())));
        via_header.uri = uri;
    }
}

fn apply_via_ttl(via_header: &mut Via, peer: &SocketAddr) {
    use rsip::common::uri::Param;

    if peer.ip().is_ipv4() {
        let mut uri = via_header.uri.clone();
        uri.params
            .push(Param::Other("ttl".into(), Some("1".into())));
        via_header.uri = uri;
    }
}

//TODO: take domain from config/yaml
fn apply_via_sent_by(via_header: &mut Via) {
    let mut uri = via_header.uri.clone();
    uri.host_with_port = default_listen_address();
    via_header.uri = uri;
}

fn default_listen_address() -> HostWithPort {
    HostWithPort::SocketAddr(SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        5060,
    ))
}
