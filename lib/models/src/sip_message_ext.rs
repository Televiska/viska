use common::rsip::prelude::*;

pub trait RequestExt {
    fn ack_request_with(&self, response: rsip::Response) -> rsip::Request;
    fn provisional_of(&self, code: impl Into<rsip::common::StatusCode>) -> rsip::Response;
}

impl RequestExt for rsip::Request {
    //TODO: should probably pass headers or just To and Route header
    fn ack_request_with(&self, response: rsip::Response) -> rsip::Request {
        use rsip::{
            common::{param::Tag, Method},
            headers::*,
            Headers,
        };

        let mut headers: Headers = Default::default();
        headers.push(
            self.call_id_header()
                .expect("call_id header")
                .clone()
                .into(),
        );
        headers.push(self.from_header().expect("from header").clone().into());

        let mut to_header = response
            .to_header()
            .expect("to header")
            .typed()
            .expect("typed to header");

        if to_header.tag().is_none() {
            to_header.with_tag(Tag::default());
        }
        headers.push(to_header.into());

        //TODO: should be only the top via header
        headers.push(self.via_header().expect("via header").clone().into());
        headers.push(self.from_header().expect("to header").clone().into());

        headers.push(
            typed::CSeq::from((
                self.cseq_header()
                    .expect("cseq header")
                    .typed()
                    .expect("typed cseq header")
                    .seq,
                Method::Ack,
            ))
            .into(),
        );
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

    fn provisional_of(&self, status_code: impl Into<rsip::common::StatusCode>) -> rsip::Response {
        use rsip::{common::param::Tag, headers::*, Headers};

        let mut headers: Headers = Default::default();
        headers.push(
            self.call_id_header()
                .expect("call_id header")
                .clone()
                .into(),
        );
        headers.push(self.from_header().expect("from header").clone().into());

        let mut to_header = self
            .to_header()
            .expect("to header")
            .typed()
            .expect("typed to header");
        to_header.with_tag(Tag::default());
        headers.push(to_header.into());

        //TODO: should be only the top via header
        headers.push(self.via_header().expect("via header").clone().into());
        headers.push(self.from_header().expect("to header").clone().into());

        headers.push(self.cseq_header().expect("cseq header").clone().into());
        headers.push(MaxForwards::default().into());

        //TODO: should take into account the Route header of response

        rsip::Response {
            status_code: status_code.into(),
            headers,
            version: Default::default(),
            body: Default::default(),
        }
    }
}
