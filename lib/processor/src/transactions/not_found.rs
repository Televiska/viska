use common::uuid::Uuid;

impl super::TransactionFSM for models::transactions::NotFound {
    fn next(&self, request: models::Request) -> Result<models::Response, String> {
        match self {
            _ => Ok(create_final_response_from(request)),
        }
    }
}

fn create_final_response_from(request: models::Request) -> models::Response {
    use common::libsip::headers::{Header, Headers};

    let mut headers = Headers::new();
    headers.push(Header::Via(
        request.via_header().expect("request Via header").clone(),
    ));
    headers.push(Header::From(
        request.from_header().expect("request From header").clone(),
    ));
    let mut to = request.to_header().expect("request To header").clone();
    to.set_param("tag", Some(format!("viska-{}", Uuid::new_v4())));
    headers.push(Header::To(to));
    headers.push(Header::CallId(
        request.call_id().expect("request CallId header").clone(),
    ));
    let cseq = request.cseq().expect("request CallId header");
    headers.push(Header::CSeq(cseq.0, cseq.1));
    headers.push(Header::ContentLength(0));
    headers.push(Header::Server("viska".into()));

    models::Response {
        code: 404,
        version: Default::default(),
        headers,
        body: vec![],
    }
}
