use crate::common::factories::prelude::*;
use rsip::{common::*, headers::*, Header, Headers};
use std::{convert::TryInto, net::IpAddr as StdIpAddr};

pub fn response(from_uri: Option<Uri>, to_uri: Option<Uri>) -> rsip::Response {
    let mut headers: Headers = Default::default();
    let from_uri = from_uri.unwrap_or_else(Uri::localhost);
    let to_uri = to_uri.unwrap_or_else(|| Uri::localhost_with_port(5090));
    headers.push(Via::from(from_uri.clone()).into());
    headers.push(From::from(from_uri.clone()).into());
    headers.push(To::from(to_uri.clone()).into());
    headers.push(CallId::default().into());
    headers.push(Contact::from(from_uri.clone()).into());
    headers.push(CSeq::from((1, Method::Invite)).into());
    headers.push(ContentLength::default().into());
    headers.push(UserAgent::default().into());

    rsip::Response {
        code: 200.into(),
        version: Version::V2,
        headers,
        body: vec![],
    }
}

pub fn trying_response_from(request: rsip::Request) -> rsip::Response {
    use rsip::{common::*, headers::*, message::HeadersExt, Headers};

    let mut headers: Headers = Default::default();
    headers.push(request.via_header().expect("via header").clone().into());
    headers.push(From(request.to_header().expect("to header").clone().0).into());
    headers.push(To(request.from_header().expect("from header").clone().0).into());
    headers.push(
        request
            .call_id_header()
            .expect("call_id header")
            .clone()
            .into(),
    );
    headers.push(
        CSeq::from((
            request.cseq_header().expect("cseq header").seq,
            request.method,
        ))
        .into(),
    );
    headers.push(MaxForwards::default().into());

    rsip::Response {
        code: StatusCode::Trying,
        headers,
        version: Default::default(),
        body: Default::default(),
    }
}

pub fn ok_response_from(request: rsip::Request) -> rsip::Response {
    use rsip::{common::*, headers::*, message::HeadersExt, Headers};

    let mut headers: Headers = Default::default();
    headers.push(request.via_header().expect("via header").clone().into());
    headers.push(From(request.to_header().expect("to header").clone().0).into());
    headers.push(To(request.from_header().expect("from header").clone().0).into());
    headers.push(
        request
            .call_id_header()
            .expect("call_id header")
            .clone()
            .into(),
    );
    headers.push(
        CSeq::from((
            request.cseq_header().expect("cseq header").seq,
            request.method,
        ))
        .into(),
    );
    headers.push(MaxForwards::default().into());

    rsip::Response {
        code: StatusCode::OK,
        headers,
        version: Default::default(),
        body: Default::default(),
    }
}

pub fn request_failure_response_from(request: rsip::Request) -> rsip::Response {
    use rsip::{common::*, headers::*, message::HeadersExt, Headers};

    let mut headers: Headers = Default::default();
    headers.push(request.via_header().expect("via header").clone().into());
    headers.push(From(request.to_header().expect("to header").clone().0).into());
    headers.push(To(request.from_header().expect("from header").clone().0).into());
    headers.push(
        request
            .call_id_header()
            .expect("call_id header")
            .clone()
            .into(),
    );
    headers.push(
        CSeq::from((
            request.cseq_header().expect("cseq header").seq,
            request.method,
        ))
        .into(),
    );
    headers.push(MaxForwards::default().into());

    rsip::Response {
        code: StatusCode::NotFound,
        headers,
        version: Default::default(),
        body: Default::default(),
    }
}

pub fn redirection_response_from(request: rsip::Request) -> rsip::Response {
    use rsip::{common::*, headers::*, message::HeadersExt, Headers};

    let mut headers: Headers = Default::default();
    headers.push(request.via_header().expect("via header").clone().into());
    headers.push(From(request.to_header().expect("to header").clone().0).into());
    headers.push(To(request.from_header().expect("from header").clone().0).into());
    headers.push(
        request
            .call_id_header()
            .expect("call_id header")
            .clone()
            .into(),
    );
    headers.push(
        CSeq::from((
            request.cseq_header().expect("cseq header").seq,
            request.method,
        ))
        .into(),
    );
    headers.push(MaxForwards::default().into());

    rsip::Response {
        code: StatusCode::MovedPermanently,
        headers,
        version: Default::default(),
        body: Default::default(),
    }
}
