use common::rsip::{self, prelude::*};

pub fn create_unauthorized_from(request: rsip::Request) -> Result<rsip::Response, crate::Error> {
    let mut headers: rsip::Headers = Default::default();
    headers.push(request.via_header()?.clone().into());
    headers.push(request.from_header()?.clone().into());
    let mut to = request.to_header()?.typed()?;
    to.with_tag(rsip::param::Tag::default());
    headers.push(to.into());
    headers.push(request.call_id_header()?.clone().into());
    headers.push(request.cseq_header()?.clone().into());
    headers.push(rsip::Header::ContentLength(Default::default()));
    headers.push(rsip::Header::Server(Default::default()));
    headers.push(rsip::Header::WwwAuthenticate(
        www_authenticate_header_value()?,
    ));

    Ok(rsip::Response {
        status_code: 401.into(),
        headers,
        ..Default::default()
    })
}

pub fn create_404_from(request: rsip::Request) -> Result<rsip::Response, crate::Error> {
    let mut headers: rsip::Headers = Default::default();
    headers.push(request.via_header()?.clone().into());
    headers.push(request.from_header()?.clone().into());
    let mut to = request.to_header()?.typed()?;
    to.with_tag(rsip::param::Tag::default());
    headers.push(to.into());
    headers.push(request.call_id_header()?.clone().into());
    headers.push(request.cseq_header()?.clone().into());
    headers.push(rsip::Header::ContentLength(Default::default()));
    headers.push(rsip::Header::Server(Default::default()));

    Ok(rsip::Response {
        headers,
        status_code: 404.into(),
        ..Default::default()
    })
}

pub fn create_405_from(request: rsip::Request) -> Result<rsip::Response, crate::Error> {
    let mut headers: rsip::Headers = Default::default();
    headers.push(request.via_header()?.clone().into());
    headers.push(request.from_header()?.clone().into());
    let mut to = request.to_header()?.clone().typed()?;
    to.with_tag(rsip::param::Tag::default());
    headers.push(to.into());
    headers.push(request.call_id_header()?.clone().into());
    headers.push(request.cseq_header()?.clone().into());
    headers.push(rsip::Header::ContentLength(Default::default()));
    headers.push(rsip::Header::Server(Default::default()));

    Ok(rsip::Response {
        headers,
        status_code: 405.into(),
        ..Default::default()
    })
}

fn www_authenticate_header_value() -> Result<rsip::headers::WwwAuthenticate, crate::Error> {
    use rsip::headers::auth;

    let nonce = store::AuthRequest::create(store::DirtyAuthRequest::default())?.nonce;

    Ok(rsip::typed::WwwAuthenticate {
        realm: "192.168.0.30".into(),
        nonce,
        algorithm: Some(auth::Algorithm::Md5),
        qop: Some(auth::Qop::Auth),
        stale: Some("FALSE".into()),
        opaque: Some("".into()),
        ..Default::default()
    }
    .into())
}

pub fn is_authorized(offer: rsip::headers::Authorization) -> Result<bool, crate::Error> {
    let offer = offer.typed()?;
    Ok(
        rsip::services::DigestGenerator::from(&offer, "123123123", &rsip::Method::Register)
            .verify(&offer.response),
    )
}
