use crate::common::factories::requests;
use ::common::bytes::Bytes;
use models::{Request, Response, SipMessage};
use sip_helpers::auth::*;
use std::convert::TryInto;

mod sip {
    pub use ::common::libsip::*;

    pub fn www_auth_header_from(headers: &Headers) -> Option<AuthHeader> {
        for h in &headers.0 {
            if let Header::WwwAuthenticate(a) = h {
                return Some(a.clone());
            }
        }
        None
    }
}

#[tokio::test]
async fn generate_digest_401() {
    crate::common::setup();

    let request: SipMessage = requests::request().into();
    let processor = ::processor::Processor::new();
    let response = processor
        .process_message(request.into())
        .await
        .expect("processor response");
    let response: Response = TryInto::<Response>::try_into(response).expect("bytes to SipMessage");

    assert_eq!(response.status_code(), 401);
    let auth_header = sip::www_auth_header_from(response.headers());
    assert!(auth_header.is_some());
    let auth_header = auth_header.expect("auth header");
    let nonce = auth_header.1.get("nonce");
    assert!(nonce.is_some());
    let nonce = nonce.expect("nonce").clone();
    let auth_request = store::AuthRequest::query()
        .nonce(Some(nonce))
        .first()
        .expect("db result");
    assert!(auth_request.is_some());
    let auth_request = auth_request.expect("auth_request");
    assert!(auth_request.consumed_at.is_none());
}

#[tokio::test]
async fn request_with_auth_succeeds() {
    crate::common::setup();

    let auth_request =
        store::AuthRequest::create(store::DirtyAuthRequest::default()).expect("db result");
    let authorization_header = sip_helpers::auth::AuthorizationHeader {
        realm: "something".into(),
        nonce: auth_request.nonce,
        opaque: None,
        algorithm: Default::default(),
        username: "vasilakisfil".into(),
        uri: "something".into(),
        cnonce: None,
        nc: None,
        response: None,
        qop: None
    };

    let request: SipMessage = requests::request().into();
    let processor = ::processor::Processor::new();
    let response = processor
        .process_message(request.into())
        .await
        .expect("processor response");
    let response: Response = TryInto::<Response>::try_into(response).expect("bytes to SipMessage");

    assert_eq!(response.status_code(), 401);
    assert!(sip::www_auth_header_from(response.headers()).is_some());
}
