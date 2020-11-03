use crate::common::factories::prelude::*;
use rsip::{common::*, headers::*, Header, Headers};
use std::{convert::TryInto, net::IpAddr as StdIpAddr};

pub fn request(from_uri: Option<Uri>, to_uri: Option<Uri>) -> rsip::Request {
    let mut headers: Headers = Default::default();
    let from_uri = from_uri.unwrap_or_else(Uri::localhost);
    let to_uri = to_uri.unwrap_or_else(|| Uri::localhost_with_port(5090));
    headers.push(Via::from(from_uri.clone()).into());
    headers.push(From::from(from_uri.clone()).into());
    headers.push(To::from(to_uri.clone()).into());
    headers.push(CallId::default().into());
    headers.push(Contact::from(from_uri.clone()).into());
    headers.push(CSeq::default().into());
    headers.push(ContentLength::default().into());
    headers.push(UserAgent::default().into());

    rsip::Request {
        method: Method::Register,
        uri: to_uri,
        version: Version::V2,
        headers,
        body: vec![],
    }
}
