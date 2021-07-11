use super::ReqProcessor;
pub use crate::{Error, SipManager};
use common::{async_trait::async_trait, rsip::prelude::*};
use models::transport::{RequestMsg, ResponseMsg};
use std::{
    any::Any,
    sync::{Arc, Weak},
};

#[derive(Debug)]
pub struct Capabilities {
    sip_manager: Weak<SipManager>,
}

#[async_trait]
impl ReqProcessor for Capabilities {
    fn new(sip_manager: Weak<SipManager>) -> Self {
        Self { sip_manager }
    }

    async fn process_incoming_request(&self, msg: RequestMsg) -> Result<(), Error> {
        apply_default_checks(&msg.sip_request)?;

        let response = create_busy_here_from(msg.sip_request.clone())?;

        self.sip_manager()
            .transport
            .send(ResponseMsg::from((response, msg.peer, msg.transport)).into())
            .await
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Capabilities {
    fn sip_manager(&self) -> Arc<SipManager> {
        self.sip_manager.upgrade().expect("sip manager is missing!")
    }
}

fn apply_default_checks(request: &rsip::Request) -> Result<(), Error> {
    has_correct_request_uri(&request.uri)?;

    Ok(())
}

fn has_correct_request_uri(request_uri: &rsip::common::Uri) -> Result<(), Error> {
    if common::CONFIG.contains_addr(&request_uri.host_with_port) {
        Ok(())
    } else {
        Err(Error::from("invalid request uri"))
    }
}

fn create_busy_here_from(request: rsip::Request) -> Result<rsip::Response, crate::Error> {
    use rsip::{common::Method, headers::*, Headers};

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
