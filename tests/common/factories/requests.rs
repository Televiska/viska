use common::libsip::{self, *};
use crate::common::factories::headers;
use std::convert::TryInto;

pub fn r_uri() -> Uri {
    Uri::sip(domain!("example.com"))
}

pub fn request() -> ::models::Request {
    let mut headers = Headers::new();
    headers.push(headers::via());
    headers.push(headers::from());
    headers.push(headers::to());
    headers.push(headers::call_id());
    headers.push(headers::contact());
    headers.push(headers::cseq(Method::Register));
    headers.push(headers::contact_length(None));
    headers.push(headers::user_agent());

    RequestGenerator::new()
        .method(Method::Register)
        .headers(headers.0)
        .uri(r_uri())
        .build()
        .expect("build request")
        .try_into()
        .expect("request generator to sip message")
}
