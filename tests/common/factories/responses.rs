use crate::common::factories::prelude::*;
use common::rsip::{self, headers::*, prelude::*, Method, Uri, Version};
use std::{convert::TryInto, net::IpAddr as StdIpAddr};

pub fn response(from_uri: Option<Uri>, to_uri: Option<Uri>) -> rsip::Response {
    let mut headers: rsip::Headers = Randomized::default();
    if let Some(from_uri) = from_uri {
        let mut typed_from_header = rsip::header_opt!(headers.iter(), Header::From)
            .expect("from header")
            .typed()
            .expect("typed from header");
        headers.unique_push(typed_from_header.with_uri(from_uri).into());
    }
    if let Some(to_uri) = to_uri {
        let mut typed_to_header = rsip::header_opt!(headers.iter(), Header::To)
            .expect("to header")
            .typed()
            .expect("typed to header");
        headers.unique_push(typed_to_header.with_uri(to_uri).into());
    }

    rsip::Response {
        status_code: 200.into(),
        version: Version::V2,
        headers,
        body: vec![],
    }
}

pub fn trying_response_from(request: rsip::Request) -> rsip::Response {
    let mut headers: rsip::Headers = Default::default();
    headers.push(request.via_header().expect("via header").clone().into());
    headers.push(From::new(request.to_header().expect("to header").clone()).into());
    headers.push(To::new(request.from_header().expect("from header").clone()).into());
    headers.push(
        request
            .call_id_header()
            .expect("call_id header")
            .clone()
            .into(),
    );
    headers.push(
        typed::CSeq::from((
            request
                .cseq_header()
                .expect("cseq header")
                .typed()
                .expect("cseq typed header")
                .seq,
            request.method,
        ))
        .into(),
    );
    headers.push(MaxForwards::default().into());

    rsip::Response {
        status_code: 100.into(),
        headers,
        version: Default::default(),
        body: Default::default(),
    }
}

pub fn ok_response_from(request: rsip::Request) -> rsip::Response {
    let mut headers: rsip::Headers = Default::default();
    headers.push(request.via_header().expect("via header").clone().into());
    headers.push(From::new(request.to_header().expect("to header").clone()).into());
    headers.push(To::new(request.from_header().expect("from header").clone()).into());
    headers.push(
        request
            .call_id_header()
            .expect("call_id header")
            .clone()
            .into(),
    );
    headers.push(
        typed::CSeq::from((
            request
                .cseq_header()
                .expect("cseq header")
                .typed()
                .expect("typed cseq header")
                .seq,
            request.method,
        ))
        .into(),
    );
    headers.push(MaxForwards::default().into());

    rsip::Response {
        status_code: 200.into(),
        headers,
        version: Default::default(),
        body: Default::default(),
    }
}

pub fn request_failure_response_from(request: rsip::Request) -> rsip::Response {
    let mut headers: rsip::Headers = Default::default();
    headers.push(request.via_header().expect("via header").clone().into());
    headers.push(From::new(request.to_header().expect("to header").clone()).into());
    headers.push(To::new(request.from_header().expect("from header").clone()).into());
    headers.push(
        request
            .call_id_header()
            .expect("call_id header")
            .clone()
            .into(),
    );
    headers.push(
        typed::CSeq::from((
            request
                .cseq_header()
                .expect("cseq header")
                .typed()
                .expect("cseq typed header")
                .seq,
            request.method,
        ))
        .into(),
    );
    headers.push(MaxForwards::default().into());

    rsip::Response {
        status_code: 404.into(),
        headers,
        version: Default::default(),
        body: Default::default(),
    }
}

pub fn redirection_response_from(request: rsip::Request) -> rsip::Response {
    let mut headers: rsip::Headers = Default::default();
    headers.push(request.via_header().expect("via header").clone().into());
    headers.push(From::new(request.to_header().expect("to header").clone()).into());
    headers.push(To::new(request.from_header().expect("from header").clone()).into());
    headers.push(
        request
            .call_id_header()
            .expect("call_id header")
            .clone()
            .into(),
    );
    headers.push(
        typed::CSeq::from((
            request
                .cseq_header()
                .expect("cseq header")
                .typed()
                .expect("cseq typed header")
                .seq,
            request.method,
        ))
        .into(),
    );
    headers.push(MaxForwards::default().into());

    rsip::Response {
        status_code: 301.into(),
        headers,
        version: Default::default(),
        body: Default::default(),
    }
}
