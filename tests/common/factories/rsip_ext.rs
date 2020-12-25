use crate::common::factories::prelude::*;
use rsip::{common::*, headers::*, Header, Headers};

impl Randomized for rsip::Request {
    fn default() -> Self {
        Self {
            method: Method::default(),
            uri: Uri::localhost(),
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

        let from_uri = Uri::localhost();
        let to_uri = Uri::localhost_with_port(5090);

        headers.push(Via::from(from_uri.clone()).into());
        headers.push(From::from(from_uri.clone()).into());
        headers.push(To::from(to_uri.clone()).into());
        headers.push(CallId::default().into());
        headers.push(Contact::from(from_uri.clone()).into());
        headers.push(CSeq::default().into());
        headers.push(ContentLength::default().into());
        headers.push(UserAgent::default().into());

        headers
    }
}

impl Randomized for rsip::Response {
    fn default() -> Self {
        Self {
            code: StatusCode::default(),
            version: Default::default(),
            headers: Randomized::default(),
            body: vec![],
        }
    }
}
