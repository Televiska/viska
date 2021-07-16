use super::ReqProcessor;
use crate::{Error, SipManager};
use common::{async_trait::async_trait, rsip::prelude::*};
use models::transport::{RequestMsg, ResponseMsg};
use std::{
    any::Any,
    sync::{Arc, Weak},
};

#[derive(Debug)]
pub struct Registrar {
    sip_manager: Weak<SipManager>,
}

#[async_trait]
impl ReqProcessor for Registrar {
    fn new(sip_manager: Weak<SipManager>) -> Self {
        Self { sip_manager }
    }

    async fn process_incoming_request(&self, msg: RequestMsg) -> Result<(), Error> {
        apply_default_checks(&msg.sip_request)?;

        match msg.sip_request.contact_header() {
            Ok(_) => self.handle_update(msg).await,
            Err(_) => self.handle_query(msg).await,
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Registrar {
    fn sip_manager(&self) -> Arc<SipManager> {
        self.sip_manager.upgrade().expect("sip manager is missing!")
    }

    async fn handle_update(&self, msg: RequestMsg) -> Result<(), Error> {
        use std::convert::TryFrom;

        for contact_header in msg.sip_request.contact_headers() {
            let typed_contact_header = contact_header.typed()?;

            match expires_value_for(contact_header, msg.sip_request.expires_header())? {
                0 => {
                    store::Registration::delete_by_uri(typed_contact_header.uri.to_string())?;
                }
                _ => {
                    println!("{:?}", typed_contact_header);
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
                .map(Into::into)
                .collect::<Vec<rsip::headers::Contact>>(),
        )?;
        self.sip_manager()
            .transport
            .send(ResponseMsg::from((response, msg.peer, msg.transport)).into())
            .await
    }
}

fn apply_default_checks(request: &rsip::Request) -> Result<(), Error> {
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
    let typed_from_header = from_header.typed()?;
    let typed_to_header = to_header.typed()?;

    if typed_from_header.uri != typed_to_header.uri {
        return Err(Error::from("mismatch between to and from header uris!"));
    }

    if typed_from_header.display_name != typed_to_header.display_name {
        return Err(Error::from(
            "mismatch between to and from header display names!",
        ));
    }

    Ok(())
}

fn has_correct_request_uri(request_uri: &rsip::common::Uri) -> Result<(), Error> {
    if common::CONFIG.contains_addr(&request_uri.host_with_port) {
        Ok(())
    } else {
        Err(Error::from("invalid request uri"))
    }
}

fn has_correct_to_request_uri(to_header: &rsip::headers::To) -> Result<(), Error> {
    let typed_to_header = to_header.typed()?;

    if common::CONFIG.contains_addr(&typed_to_header.uri.host_with_port) {
        Ok(())
    } else {
        Err(Error::from("record not found!"))
    }
}

fn extensions_are_supported() -> Result<(), Error> {
    Ok(())
}

fn create_registration_ok_from(
    request: rsip::Request,
    contacts: Vec<rsip::headers::Contact>,
) -> Result<rsip::Response, crate::Error> {
    use rsip::{common::Method, headers::*, Headers};

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
    let mut typed_to_header = request.to_header()?.typed()?;
    typed_to_header.with_tag(Default::default());
    headers.push(typed_to_header.into());
    headers.push(ContentLength::default().into());
    headers.push(Server::default().into());
    for contact in contacts {
        headers.push(contact.into());
    }

    Ok(rsip::Response {
        status_code: 200.into(),
        headers,
        ..Default::default()
    })
}

fn expires_value_for(
    contact_header: &rsip::headers::Contact,
    expires_header: Option<&rsip::headers::Expires>,
) -> Result<u32, Error> {
    let typed_contact_header = contact_header.typed()?;

    match typed_contact_header.expires() {
        Some(expire) => Ok(expire.seconds()?),
        _ => match expires_header {
            Some(header) => Ok(header.seconds()?),
            _ => Ok(600),
        },
    }
}
