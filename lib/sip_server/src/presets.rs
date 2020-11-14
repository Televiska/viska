use common::uuid::Uuid;
use rsip::{
    headers::{Authorization, Headers, WwwAuthenticate},
    message::HeadersExt,
    Request, Response,
};

pub fn create_registration_ok_from(request: Request) -> Result<Response, crate::Error> {
    use rsip::{
        common::Method,
        headers::{ContactParam, Header, NamedParam},
    };

    let mut headers: Headers = Default::default();
    headers.push(request.via_header()?.clone().into());
    headers.push(request.from_header()?.clone().into());
    let mut to = request.to_header()?.clone();
    to.0.add_param(NamedParam::Tag(Default::default()));
    headers.push(to.into());
    headers.push(request.call_id_header()?.clone().into());
    headers.push(request.cseq_header()?.clone().into());
    if let Method::Register = request.method() {
        let mut contact = request.contact_header()?.clone();
        contact
            .0
            .add_param(ContactParam::Custom("expires".into(), Some("600".into())));
        headers.push(contact.into());
    }
    headers.push(Header::ContentLength(Default::default()));
    headers.push(Header::Server(Default::default()));

    Ok(Response {
        code: 200.into(),
        headers,
        ..Default::default()
    })
}

pub fn create_unauthorized_from(request: Request) -> Result<Response, crate::Error> {
    use rsip::headers::{Header, NamedParam};

    let mut headers: Headers = Default::default();
    headers.push(request.via_header()?.clone().into());
    headers.push(request.from_header()?.clone().into());
    let mut to = request.to_header()?.clone();
    to.0.add_param(NamedParam::Tag(Default::default()));
    headers.push(to.into());
    headers.push(request.call_id_header()?.clone().into());
    headers.push(request.cseq_header()?.clone().into());
    headers.push(Header::ContentLength(Default::default()));
    headers.push(Header::Server(Default::default()));
    headers.push(Header::WwwAuthenticate(www_authenticate_header_value()?));

    Ok(Response {
        code: 401.into(),
        headers,
        ..Default::default()
    })
}

pub fn create_404_from(request: Request) -> Result<Response, crate::Error> {
    use rsip::headers::{Header, NamedParam};

    let mut headers: Headers = Default::default();
    headers.push(request.via_header()?.clone().into());
    headers.push(request.from_header()?.clone().into());
    let mut to = request.to_header()?.clone();
    to.0.add_param(NamedParam::Tag(Default::default()));
    headers.push(to.into());
    headers.push(request.call_id_header()?.clone().into());
    headers.push(request.cseq_header()?.clone().into());
    headers.push(Header::ContentLength(Default::default()));
    headers.push(Header::Server(Default::default()));

    Ok(Response {
        headers,
        code: 404.into(),
        ..Default::default()
    })
}

fn www_authenticate_header_value() -> Result<WwwAuthenticate, crate::Error> {
    //let nonce = store::AuthRequest::create(store::DirtyAuthRequest::default())?.nonce;
    let nonce = Uuid::new_v4().to_string();
    Ok(WwwAuthenticate::new("192.168.1.223".into(), nonce))
}

pub fn is_authorized(offer: Authorization) -> Result<bool, crate::Error> {
    Ok(offer.verify_for("123123123".into())?)
}
