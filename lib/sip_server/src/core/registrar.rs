pub use crate::{Error, SipManager};
use models::transport::{RequestMsg, ResponseMsg};
use rsip::common::uri::HostWithPort;
use std::{
    convert::TryInto,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::{Arc, Weak},
};

#[derive(Debug)]
pub struct Registrar {
    sip_manager: Weak<SipManager>,
}

#[allow(clippy::new_without_default)]
impl Registrar {
    pub fn new(sip_manager: Weak<SipManager>) -> Self {
        Self { sip_manager }
    }

    pub async fn process_incoming_request(&self, msg: RequestMsg) -> Result<(), Error> {
        use rsip::message::HeadersExt;

        apply_default_checks(&msg.sip_request)?;

        match msg.sip_request.contact_header() {
            Ok(_) => self.handle_update(msg).await,
            Err(_) => self.handle_query(msg).await,
        }
    }

    fn sip_manager(&self) -> Arc<SipManager> {
        self.sip_manager.upgrade().expect("sip manager is missing!")
    }

    async fn handle_update(&self, msg: RequestMsg) -> Result<(), Error> {
        use rsip::message::{ExpiresExt, HeadersExt};
        use std::convert::TryFrom;

        for contact_header in msg.sip_request.contact_headers() {
            match expires_value_for(contact_header, msg.sip_request.expires_header()) {
                0 => {
                    store::Registration::delete_by_uri(contact_header.0.uri.to_string())?;
                }
                _ => {
                    store::Registration::upsert(store::DirtyRegistration::try_from(msg.clone())?)?;
                }
            }
        }

        self.handle_query(msg).await
    }

    async fn handle_query(&self, msg: RequestMsg) -> Result<(), Error> {
        let response = create_registration_ok_from(
            msg.sip_request.clone(),
            store::Registration::search(Default::default())?
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<rsip::headers::Contact>, rsip::Error>>()?,
        )?;
        self.sip_manager()
            .transport
            .send(ResponseMsg::from((response, msg.peer, msg.transport)).into())
            .await
    }
}

fn apply_default_checks(request: &rsip::Request) -> Result<(), Error> {
    use rsip::message::HeadersExt;

    let to_header = request.to_header()?;
    let from_header = request.from_header()?;

    has_correct_request_uri(&request.uri)?;
    extensions_are_supported()?;
    has_correct_to_request_uri(to_header)?;
    has_same_from_to_header_uris(from_header, to_header)?;

    Ok(())
}

fn has_same_from_to_header_uris(
    from_header: &rsip::headers::From,
    to_header: &rsip::headers::To,
) -> Result<(), Error> {
    if from_header.0.uri != to_header.0.uri {
        return Err(Error::from("mismatch between to and from header uris!"));
    }

    if from_header.0.display_name != to_header.0.display_name {
        return Err(Error::from(
            "mismatch between to and from header display names!",
        ));
    }

    Ok(())
}

fn has_correct_request_uri(request_uri: &rsip::common::Uri) -> Result<(), Error> {
    if request_uri.host_with_port == default_request_uri() {
        Ok(())
    } else {
        Err(Error::from("invalid request uri"))
    }
}

fn has_correct_to_request_uri(to_header: &rsip::headers::To) -> Result<(), Error> {
    if to_header.0.uri.host_with_port == default_request_uri() {
        Ok(())
    } else {
        Err(Error::from("record not found!"))
    }
}

fn extensions_are_supported() -> Result<(), Error> {
    Ok(())
}

fn default_request_uri() -> HostWithPort {
    HostWithPort::SocketAddr(SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        5060,
    ))
}

fn create_registration_ok_from(
    request: rsip::Request,
    contacts: Vec<rsip::headers::Contact>,
) -> Result<rsip::Response, crate::Error> {
    use rsip::{
        common::Method,
        headers::{Header, NamedParam},
        message::HeadersExt,
        Headers,
    };

    if *request.method() != Method::Register {
        return Err(crate::Error::custom(format!(
            "must have REGISTER method, found: {}",
            request.method()
        )));
    }

    let mut headers: Headers = Default::default();
    headers.push(request.from_header()?.clone().into());
    headers.push(request.call_id_header()?.clone().into());
    headers.push(request.cseq_header()?.clone().into());
    headers.push(request.via_header()?.clone().into());
    let mut to = request.to_header()?.clone();
    to.0.add_param(NamedParam::Tag(Default::default()));
    headers.push(to.into());
    headers.push(Header::ContentLength(Default::default()));
    headers.push(Header::Server(Default::default()));
    for contact in contacts {
        headers.push(contact.into());
    }

    Ok(rsip::Response {
        code: 200.into(),
        headers,
        ..Default::default()
    })
}

fn expires_value_for(
    contact_header: &rsip::headers::Contact,
    expires_header: Result<&rsip::headers::Expires, rsip::Error>,
) -> u32 {
    match contact_header.expires() {
        Ok(Some(expire)) => expire,
        _ => match expires_header {
            Ok(header) => header.0,
            _ => 600,
        },
    }
}
