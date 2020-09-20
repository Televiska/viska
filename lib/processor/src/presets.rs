use common::{
    libsip::{
        core::method::Method,
        headers::{AuthHeader, Header, Headers},
        uri::{Domain, UriParam},
        ResponseGenerator,
    },
    uuid::Uuid,
};
use sip_helpers::auth::{AuthorizationHeader, WwwAuthenticateHeader};
use std::convert::TryInto;
use std::net::Ipv4Addr;

pub fn create_registration_ok_from(
    request: models::Request,
) -> Result<models::Response, crate::Error> {
    let mut headers = Headers::new();
    let mut via_header = request.via_header()?.clone();
    let uri = via_header.uri.clone();
    let uri = uri.parameters(vec![
        UriParam::RPort(Some(5066)),
        UriParam::Branch(request.via_header_branch()?.clone()),
        UriParam::Received(Domain::Ipv4(Ipv4Addr::new(192, 168, 1, 223), None)),
    ]);
    via_header.uri = uri;
    headers.push(Header::Via(via_header));
    headers.push(Header::From(request.from_header()?.clone()));
    let mut to = request.to_header()?.clone();
    to.set_param("tag", Some(format!("viska-{}", Uuid::new_v4())));
    headers.push(Header::To(to));
    headers.push(Header::CallId(request.call_id()?.clone()));
    let cseq = request.cseq()?;
    headers.push(Header::CSeq(cseq.0, cseq.1));

    if let Method::Register = request.method() {
        let mut contact = request.contact_header()?.clone();
        contact.set_param("expires", Some("600"));
        headers.push(Header::Contact(contact));
    }
    headers.push(Header::ContentLength(0));
    headers.push(Header::Server("viska".into()));

    Ok(ResponseGenerator::new()
        .code(200)
        .headers(headers.0)
        .build()?
        .try_into()?)
}

pub fn create_unauthorized_from(
    request: models::Request,
) -> Result<models::Response, crate::Error> {
    let mut headers = Headers::new();
    let mut via_header = request.via_header()?.clone();
    let uri = via_header.uri.clone();
    let uri = uri.parameters(vec![
        UriParam::RPort(Some(5066)),
        UriParam::Branch(request.via_header_branch()?.clone()),
        UriParam::Received(Domain::Ipv4(Ipv4Addr::new(192, 168, 1, 223), None)),
    ]);
    via_header.uri = uri;
    headers.push(Header::Via(via_header));
    headers.push(Header::From(request.from_header()?.clone()));
    let mut to = request.to_header()?.clone();
    to.set_param("tag", Some(format!("viska-{}", Uuid::new_v4())));
    headers.push(Header::To(to));
    headers.push(Header::CallId(request.call_id()?.clone()));
    let cseq = request.cseq()?;
    headers.push(Header::CSeq(cseq.0, cseq.1));
    if let Method::Register = request.method() {
        let mut contact = request.contact_header()?.clone();
        contact.set_param("expires", Some("600"));
        headers.push(Header::Contact(contact));
    }
    headers.push(Header::ContentLength(0));
    headers.push(Header::Server("viska".into()));
    headers.push(Header::WwwAuthenticate(www_authenticate_header_value()?));

    Ok(ResponseGenerator::new()
        .code(401)
        .headers(headers.0)
        .build()?
        .try_into()?)
}

pub fn create_404_from(request: models::Request) -> Result<models::Response, crate::Error> {
    let mut headers = Headers::new();
    headers.push(Header::Via(request.via_header()?.clone()));
    headers.push(Header::From(request.from_header()?.clone()));
    let mut to = request.to_header()?.clone();
    to.set_param("tag", Some(format!("viska-{}", Uuid::new_v4())));
    headers.push(Header::To(to));
    headers.push(Header::CallId(request.call_id()?.clone()));
    let cseq = request.cseq()?;
    headers.push(Header::CSeq(cseq.0, cseq.1));
    headers.push(Header::ContentLength(0));
    headers.push(Header::Server("viska".into()));

    Ok(ResponseGenerator::new()
        .code(404)
        .headers(headers.0)
        .build()?
        .try_into()?)
}

fn www_authenticate_header_value() -> Result<AuthHeader, crate::Error> {
    let nonce = store::AuthRequest::create(store::DirtyAuthRequest::default())?.nonce;
    let header = WwwAuthenticateHeader::new("192.168.1.223".into(), nonce);

    Ok(header.into())
}

pub fn is_authorized(offer: AuthorizationHeader) -> Result<bool, crate::Error> {
    Ok(offer.verify_for("123123123".into())?)
}
