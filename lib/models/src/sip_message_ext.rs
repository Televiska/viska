use crate::Error;
use rsip::message::HeadersExt;

pub trait SipMessageExt {
    fn transaction_id(&self) -> Result<String, Error>;
}

impl SipMessageExt for rsip::Request {
    fn transaction_id(&self) -> Result<String, Error> {
        Ok(self
            .via_header()?
            .branch()
            .ok_or("missing branch in via header!")?.clone().into())
    }
}

impl SipMessageExt for rsip::Response {
    fn transaction_id(&self) -> Result<String, Error> {
        Ok(self
            .via_header()?
            .branch()
            .ok_or("missing branch in via header!")?.clone().into())
    }
}

impl SipMessageExt for rsip::SipMessage {
    fn transaction_id(&self) -> Result<String, Error> {
        match self {
            Self::Request(request) => request.transaction_id(),
            Self::Response(response) => response.transaction_id(),
        }
    }
}

pub trait RequestExt {
    fn ack_request_with(&self, response: rsip::Response) -> rsip::Request;
    fn provisional_of(&self, code: impl Into<rsip::common::StatusCode>) -> rsip::Response;
}

impl RequestExt for rsip::Request {
    //TODO: should probably pass headers or just To and Route header
    fn ack_request_with(&self, response: rsip::Response) -> rsip::Request {
        use rsip::{common::*, headers::*, Headers};

        let mut headers: Headers = Default::default();
        headers.push(
            self.call_id_header()
                .expect("call_id header")
                .clone()
                .into(),
        );
        headers.push(self.from_header().expect("from header").clone().into());

        let mut to_header = response.to_header().expect("from header").clone();
        if to_header.tag().is_none() {
            to_header.with_tag(named::Tag::default());
        }
        headers.push(to_header.into());

        //TODO: should be only the top via header
        headers.push(self.via_header().expect("via header").clone().into());
        headers.push(self.from_header().expect("to header").clone().into());

        headers
            .push(CSeq::from((self.cseq_header().expect("cseq header").seq, Method::Ack)).into());
        headers.push(MaxForwards::default().into());

        //TODO: should take into account the Route header of response

        rsip::Request {
            method: Method::Ack,
            uri: self.uri.clone(),
            headers,
            version: Default::default(),
            body: Default::default(),
        }
    }

    fn provisional_of(&self, code: impl Into<rsip::common::StatusCode>) -> rsip::Response {
        use rsip::{headers::*, Headers};

        let mut headers: Headers = Default::default();
        headers.push(
            self.call_id_header()
                .expect("call_id header")
                .clone()
                .into(),
        );
        headers.push(self.from_header().expect("from header").clone().into());

        let mut to_header = self.to_header().expect("from header").clone();
        to_header.with_tag(named::Tag::default());
        headers.push(to_header.into());

        //TODO: should be only the top via header
        headers.push(self.via_header().expect("via header").clone().into());
        headers.push(self.from_header().expect("to header").clone().into());

        headers.push(self.cseq_header().expect("cseq header").clone().into());
        headers.push(MaxForwards::default().into());

        //TODO: should take into account the Route header of response

        rsip::Response {
            code: code.into(),
            headers,
            version: Default::default(),
            body: Default::default(),
        }
    }
}
