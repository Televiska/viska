use common::rsip::{self, prelude::*};

pub trait RequestExt {
    fn ack_request_from(&self, response: rsip::Response) -> rsip::Request;
    fn provisional_of(&self, code: impl Into<rsip::StatusCode>) -> rsip::Response;
}

impl RequestExt for rsip::Request {
    fn ack_request_from(&self, response: rsip::Response) -> rsip::Request {
        use rsip::{headers::*, Headers, Method};

        let mut headers: Headers = Default::default();
        headers.push(
            self.call_id_header()
                .expect("call_id header")
                .clone()
                .into(),
        );
        headers.push(self.from_header().expect("from header").clone().into());

        let to_header = response
            .to_header()
            .expect("to header")
            .typed()
            .expect("typed to header");

        headers.push(to_header.with_tag(Default::default()).into());

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

    fn provisional_of(&self, status_code: impl Into<rsip::StatusCode>) -> rsip::Response {
        use rsip::{headers::*, Headers};

        let mut headers: Headers = Default::default();
        headers.push(
            self.call_id_header()
                .expect("call_id header")
                .clone()
                .into(),
        );
        headers.push(self.from_header().expect("from header").clone().into());

        let to_header = self
            .to_header()
            .expect("to header")
            .typed()
            .expect("typed to header");
        headers.push(to_header.with_tag(Default::default()).into());

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
