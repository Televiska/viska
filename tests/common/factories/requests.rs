use crate::common::factories::prelude::*;
use rsip::{common::*, headers::*, Header, Headers};
use std::{convert::TryInto, net::IpAddr as StdIpAddr};

pub fn request(from_uri: Option<Uri>, to_uri: Option<Uri>) -> rsip::Request {
    let mut headers: Headers = Randomized::default();
    if let Some(from_uri) = from_uri {
        let mut from_header: From = rsip::header_opt!(headers.iter(), Header::From)
            .expect("from header")
            .clone();
        headers.unique_push(from_header.with_uri(from_uri).into());
    }
    if let Some(to_uri) = to_uri {
        let mut to_header: To = rsip::header_opt!(headers.iter(), Header::To)
            .expect("to header")
            .clone();
        headers.unique_push(to_header.with_uri(to_uri).into());
    }
    let to_header: To = rsip::header_opt!(headers.iter(), Header::To)
        .expect("to header")
        .clone();

    rsip::Request {
        method: Method::Register,
        uri: to_header.0.uri.stripped(),
        version: Version::V2,
        headers,
        body: vec![],
    }
}

pub fn invite_request() -> rsip::Request {
    let mut headers: Headers = Randomized::default();
    headers.unique_push(CSeq::from((1, Method::Invite)).into());
    let mut to_header: To = rsip::header_opt!(headers.iter(), Header::To)
        .expect("to header")
        .clone();

    rsip::Request {
        method: Method::Invite,
        uri: to_header.0.uri.stripped(),
        headers,
        ..Randomized::default()
    }
}

pub fn register_query_request() -> rsip::Request {
    let mut headers: Headers = Randomized::default();
    headers.unique_push(CSeq::from((1, Method::Register)).into());

    let base_uri = Uri::default().sips();
    let from_uri = base_uri.clone().with_username("filippos");
    let to_uri = from_uri.clone();

    headers.unique_push(From::from(from_uri).into());
    headers.unique_push(To::from(to_uri.clone()).into());
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

    let from_header = rsip::header_opt!(headers.iter(), Header::From)
        .expect("from header")
        .clone();
    headers.unique_push(Contact::from(from_header.0.uri.clone()).into());

    rsip::Request {
        method: Method::Register,
        uri: from_header.0.uri.stripped().sips(),
        headers,
        ..Randomized::default()
    }
}

pub fn register_delete_request_with_uri(uri: Uri) -> rsip::Request {
    let request = register_query_request();
    let mut headers = request.headers.clone();

    let from_header = rsip::header_opt!(headers.iter(), Header::From)
        .expect("from header")
        .clone();
    headers.unique_push(Contact::from(uri).into());
    headers.unique_push(Expires::from(0).into());

    rsip::Request {
        method: Method::Register,
        uri: from_header.0.uri.stripped().sips(),
        headers,
        ..Randomized::default()
    }
}

pub fn options_request() -> rsip::Request {
    let mut headers: Headers = Randomized::default();
    headers.unique_push(CSeq::from((1, Method::Options)).into());

    let base_uri = Uri::default().sips();
    let to_uri = base_uri.clone().with_username("filippos");

    headers.unique_push(To::from(to_uri.clone()).into());
    headers.retain(|h| !matches!(h, Header::Contact(_)));

    rsip::Request {
        method: Method::Options,
        uri: base_uri,
        headers,
        ..Randomized::default()
    }
}
