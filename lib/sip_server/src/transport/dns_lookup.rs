use super::DnsLookup;
use crate::Error;
use common::{
    async_trait::async_trait,
    rsip::{self, headers::ToTypedHeader, message::HeadersExt},
    rsip_dns,
};
use models::transport::{RequestMsg, ResponseMsg};

pub struct DefaultDnsLookup;

#[async_trait]
impl DnsLookup for DefaultDnsLookup {
    async fn request_msg_from(&self, request: rsip::Request) -> Result<RequestMsg, Error> {
        let target = resolve_address_for(request.uri.clone()).await;

        Ok(RequestMsg {
            sip_request: request,
            peer: target.socket_addr(),
            transport: rsip::Transport::Udp,
        })
    }

    async fn response_msg_from(&self, response: rsip::Response) -> Result<ResponseMsg, Error> {
        let via_header = response.via_header()?.typed()?;
        let port: u16 = via_header
            .sent_by()
            .port()
            .cloned()
            .map(Into::into)
            .unwrap_or(5060);

        match (
            via_header.sent_protocol(),
            via_header.received().ok().flatten(),
        ) {
            (rsip::Transport::Udp, Some(received)) => Ok(ResponseMsg {
                sip_response: response,
                peer: (received, port).into(),
                transport: rsip::Transport::Udp,
            }),
            (rsip::Transport::Udp, None) => match via_header.sent_by().host() {
                rsip::Host::Domain(_) => panic!("need to run from RFC3263"),
                rsip::Host::IpAddr(ip_addr) => Ok(ResponseMsg {
                    sip_response: response,
                    peer: (*ip_addr, port).into(),
                    transport: rsip::Transport::Udp,
                }),
            },
            (transport, _) => panic!("not supported transport: {}", transport),
        }
    }
}

//TODO: atm only one (the first) target is supported
async fn resolve_address_for(uri: rsip::Uri) -> rsip_dns::Target {
    use rsip_dns::{trust_dns_resolver::TokioAsyncResolver, ResolvableExt};

    let context = rsip_dns::Context::initialize_from(
        uri,
        rsip_dns::AsyncTrustDnsClient::new(
            TokioAsyncResolver::tokio(Default::default(), Default::default()).unwrap(),
        ),
        rsip_dns::SupportedTransports::only(vec![rsip::Transport::Udp]),
    )
    .unwrap();

    let mut lookup = rsip_dns::Lookup::from(context);

    lookup
        .resolve_next()
        .await
        .expect("next Target in dns lookup")
}
