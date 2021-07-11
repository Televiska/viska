use crate::common::factories::prelude::*;
use common::rsip::prelude::*;
use rsip::{
    common::{Method, Uri},
    headers::*,
};

impl Randomized for rsip::Request {
    fn default() -> Self {
        Self {
            method: Method::default(),
            uri: common::CONFIG.default_addr().into(),
            version: Default::default(),
            headers: Randomized::default(),
            body: vec![],
        }
    }
}

impl Randomized for Method {
    fn default() -> Self {
        Method::Register
    }
}

impl Randomized for Headers {
    fn default() -> Self {
        let mut headers: Headers = Default::default();

        let base_uri: Uri = common::CONFIG.default_addr().into();

        let from_uri = base_uri.clone().with_username("filippos");
        let to_uri = base_uri.clone().with_username("fil").with_port(5090);

        headers.push(to::typed::To::from(to_uri.clone()).into());
        headers.push(from::typed::From::from(from_uri.clone()).into());
        headers.push(
            cseq::typed::CSeq {
                seq: 1,
                method: Method::Register,
            }
            .into(),
        );
        headers.push(CallId::default().into());
        headers.push(MaxForwards::default().into());
        headers.push(via::typed::Via::from(base_uri.clone().stripped()).into());
        headers.push(ContentLength::default().into());
        headers.push(UserAgent::default().into());

        headers
    }
}

impl Randomized for rsip::Response {
    fn default() -> Self {
        Self {
            status_code: Default::default(),
            version: Default::default(),
            headers: Randomized::default(),
            body: vec![],
        }
    }
}
