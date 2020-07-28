use common::uuid::Uuid;

impl super::TransactionFSM for models::transactions::NotFound {
    fn next(&self, request: models::Request) -> Result<models::Response, crate::Error> {
        match self {
            _ => Ok(create_final_response_from(request)?),
        }
    }
}

fn create_final_response_from(request: models::Request) -> Result<models::Response, crate::Error> {
    use common::libsip::{
        headers::{Header, Headers},
        ResponseGenerator,
    };
    use std::convert::TryInto;

    let mut headers = Headers::new();
    headers.push(Header::Via(request.via_header()?.clone()));
    headers.push(Header::From(request.from_header()?.clone()));
    let mut to = request.to_header()?.clone();
    to.set_param("tag", Some(format!("viska-{}", Uuid::new_v4())));
    headers.push(Header::To(to));
    headers.push(Header::CallId(request.call_id()?.clone()));
    let cseq = request.cseq()?;
    headers.push(Header::CSeq(cseq.0, cseq.1));
    headers.push(Header::ContentLength(0));
    headers.push(Header::Server("viska".into()));

    Ok(ResponseGenerator::new()
        .code(404)
        .headers(headers.0)
        .build()?
        .try_into()?)
}
