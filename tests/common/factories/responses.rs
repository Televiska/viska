use crate::common::factories::{
    common::uri::{HostWithPort, TestsUriExt, Uri},
    common::Method,
    headers::{self, NamedHeader, NamedParam},
};
use common::libsip::{Headers, ResponseGenerator};
use std::{convert::TryInto, net::IpAddr as StdIpAddr};

pub fn response(from_uri: Option<Uri>, to_uri: Option<Uri>) -> ::models::Response {
    let mut headers = Headers::new();
    let from_uri = from_uri.unwrap_or_else(Uri::localhost);
    let to_uri = to_uri.unwrap_or_else(|| Uri::localhost_with_port(5090));
    headers.push(
        headers::Via {
            uri: from_uri.clone(),
            ..Default::default()
        }
        .into(),
    );
    headers.push(
        headers::From(NamedHeader {
            uri: from_uri,
            ..Default::default()
        })
        .into(),
    );
    headers.push(
        headers::To(NamedHeader {
            uri: to_uri,
            params: NamedParam::default().into(),
            ..Default::default()
        })
        .into(),
    );
    headers.push(headers::CallId::default().into());
    headers.push(
        headers::Contact(NamedHeader {
            uri: Uri::localhost(),
            ..Default::default()
        })
        .into(),
    );
    headers.push(headers::CSeq::default().into());
    headers.push(headers::ContentLength::default().into());
    headers.push(headers::UserAgent::default().into());

    ResponseGenerator::new()
        .headers(headers.0)
        .code(200)
        .build()
        .expect("build request")
        .try_into()
        .expect("request generator to sip message")
}
