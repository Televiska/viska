use crate::{Error, ReqProcessor};
use common::{
    async_trait::async_trait,
    rsip::{self, prelude::*},
};
use models::Handlers;

#[derive(Debug)]
pub struct Registrar {
    handlers: Handlers,
}

#[async_trait]
impl ReqProcessor for Registrar {
    async fn process_incoming_request(&self, msg: rsip::Request) -> Result<(), Error> {
        apply_default_checks(&msg)?;

        match msg.contact_header() {
            Ok(_) => self.handle_update(msg).await,
            Err(_) => self.handle_query(msg).await,
        }
    }
}

impl Registrar {
    pub fn new(handlers: Handlers) -> Self {
        Self { handlers }
    }

    async fn handle_update(&self, msg: rsip::Request) -> Result<(), Error> {
        use std::convert::TryFrom;

        for contact_header in msg.contact_headers() {
            let typed_contact_header = contact_header.typed()?;

            match expires_value_for(contact_header, msg.expires_header())? {
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

    async fn handle_query(&self, msg: rsip::Request) -> Result<(), Error> {
        let response = create_registration_ok_from(
            msg.clone(),
            store::Registration::search(Default::default())?
                .into_iter()
                .map(Into::into)
                .collect::<Vec<rsip::headers::Contact>>(),
        )?;
        Ok(self.handlers.transport.send(response.into()).await?)
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

fn has_correct_request_uri(request_uri: &rsip::Uri) -> Result<(), Error> {
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
    use rsip::{headers::*, Headers, Method};

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
    headers.push(
        request
            .to_header()?
            .typed()?
            .with_tag(Default::default())
            .into(),
    );
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
