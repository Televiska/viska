pub use crate::{Error, ReqProcessor};
use common::{
    async_trait::async_trait,
    rsip::{self, prelude::*},
};
use models::{Handlers, transport::{RequestMsg, ResponseMsg}};
use std::{
    any::Any,
    sync::{Arc, Weak},
};

#[derive(Debug)]
pub struct Capabilities {
    handlers: Handlers,
}

impl Capabilities {
    pub fn new(handlers: Handlers) -> Self {
        Self { handlers }
    }
}

#[async_trait]
impl ReqProcessor for Capabilities {
    async fn process_incoming_request(&self, msg: RequestMsg) -> Result<(), Error> {
        apply_default_checks(&msg.sip_request)?;

        let response = create_busy_here_from(msg.sip_request.clone())?;

        Ok(self.handlers
            .transport
            .send(ResponseMsg::from((response, msg.peer, msg.transport)).into())
            .await?)
    }
}

fn apply_default_checks(request: &rsip::Request) -> Result<(), Error> {
    has_correct_request_uri(&request.uri)?;

    Ok(())
}

fn has_correct_request_uri(request_uri: &rsip::Uri) -> Result<(), Error> {
    if common::CONFIG.contains_addr(&request_uri.host_with_port) {
        Ok(())
    } else {
        Err(Error::from("invalid request uri"))
    }
}

fn create_busy_here_from(request: rsip::Request) -> Result<rsip::Response, crate::Error> {
    use rsip::{headers::*, Headers, Method};

    if *request.method() != Method::Options {
        return Err(crate::Error::custom(format!(
            "must have OPTIONS method, found: {}",
            request.method()
        )));
    }

    let mut headers: Headers = Default::default();

    let mut typed_to_header = request.to_header()?.typed()?;
    typed_to_header.with_tag(Default::default());

    headers.push(request.via_header()?.clone().into());
    headers.push(typed_to_header.into());
    headers.push(request.from_header()?.clone().into());
    headers.push(request.call_id_header()?.clone().into());
    headers.push(request.cseq_header()?.clone().into());
    headers.push(ContentLength::default().into());
    headers.push(Server::default().into());
    headers.push(Allow::default().into());
    headers.push(Accept::new("application/sdp").into());
    headers.push(AcceptEncoding::new("gzip").into());
    headers.push(AcceptLanguage::new("english").into());

    Ok(rsip::Response {
        status_code: 486.into(),
        headers,
        ..Default::default()
    })
}
