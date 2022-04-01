use crate::common::factories::prelude::*;
use common::rsip::{self, headers::*, Method, Uri, Version};

pub fn request(from_uri: Option<Uri>, to_uri: Option<Uri>) -> rsip::Request {
    let mut headers: Headers = Randomized::default();
    if let Some(from_uri) = from_uri {
        let typed_from_header = rsip::header_opt!(headers.iter(), Header::From)
            .expect("from header")
            .typed()
            .expect("typed from header");
        headers.unique_push(typed_from_header.with_uri(from_uri).into());
    }
    if let Some(to_uri) = to_uri {
        let typed_to_header = rsip::header_opt!(headers.iter(), Header::To)
            .expect("to header")
            .typed()
            .expect("typed to header");
        headers.unique_push(typed_to_header.with_uri(to_uri).into());
    }
    let to_header = rsip::header_opt!(headers.iter(), Header::To)
        .expect("to header")
        .typed()
        .expect("typed to header");

    rsip::Request {
        method: Method::Register,
        uri: to_header.uri.stripped(),
        version: Version::V2,
        headers,
        body: vec![],
    }
}

pub fn invite_request() -> rsip::Request {
    let mut headers: Headers = Randomized::default();
    headers.unique_push(typed::CSeq::from((1, Method::Invite)).into());

    let typed_to_header = rsip::header_opt!(headers.iter(), Header::To)
        .unwrap()
        .typed()
        .unwrap();

    rsip::Request {
        method: Method::Invite,
        uri: typed_to_header.uri.stripped(),
        headers,
        ..Randomized::default()
    }
}

pub fn bye_request() -> rsip::Request {
    let mut headers: Headers = Randomized::default();
    headers.unique_push(typed::CSeq::from((1, Method::Bye)).into());

    let typed_to_header = rsip::header_opt!(headers.iter(), Header::To)
        .unwrap()
        .typed()
        .unwrap();

    rsip::Request {
        method: Method::Bye,
        uri: typed_to_header.uri.stripped(),
        headers,
        ..Randomized::default()
    }
}

pub fn register_query_request() -> rsip::Request {
    let mut headers: Headers = Randomized::default();
    headers.unique_push(typed::CSeq::from((1, Method::Register)).into());

    let base_uri: Uri = common::CONFIG.default_addr().into();
    let from_uri = base_uri.clone().with_user("filippos");
    let to_uri = from_uri.clone();

    headers.unique_push(typed::From::from(from_uri).into());
    headers.unique_push(typed::To::from(to_uri.clone()).into());
    headers.retain(|h| !matches!(h, Header::Contact(_)));

    rsip::Request {
        method: Method::Register,
        uri: base_uri,
        headers,
        ..Randomized::default()
    }
}

pub fn register_request() -> rsip::Request {
    let request = register_query_request();
    let mut headers = request.headers.clone();

    let from_header = rsip::header_opt!(headers.iter(), Header::From).expect("from header");
    let typed_from_header = from_header.typed().expect("typed from header");
    headers.unique_push(typed::Contact::from(typed_from_header.uri.clone()).into());

    rsip::Request {
        method: Method::Register,
        uri: typed_from_header.uri.stripped(),
        headers,
        ..Randomized::default()
    }
}

pub fn register_delete_request_with_uri(uri: Uri) -> rsip::Request {
    let request = register_query_request();
    let mut headers = request.headers.clone();

    let from_header = rsip::header_opt!(headers.iter(), Header::From).expect("from header");
    let typed_from_header = from_header.typed().expect("typed from header");
    headers.unique_push(typed::Contact::from(uri).into());
    headers.unique_push(Expires::new("0").into());

    rsip::Request {
        method: Method::Register,
        uri: typed_from_header.uri.stripped(),
        headers,
        ..Randomized::default()
    }
}

pub fn options_request() -> rsip::Request {
    let mut headers: Headers = Randomized::default();
    headers.unique_push(typed::CSeq::from((1, Method::Options)).into());

    let base_uri: Uri = common::CONFIG.default_addr().into();
    let to_uri = base_uri.clone().with_user("filippos");

    headers.unique_push(typed::To::from(to_uri.clone()).into());
    headers.retain(|h| !matches!(h, Header::Contact(_)));

    rsip::Request {
        method: Method::Options,
        uri: base_uri,
        headers,
        ..Randomized::default()
    }
}
