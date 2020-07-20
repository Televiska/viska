//use common::libsip::headers::Headers;
use crate::Response;
use common::uuid::Uuid;

#[derive(Debug, Clone)]
pub struct TransactionData {
    pub branch_id: String,
    pub dialog_id: i64,
}

#[derive(Debug, Clone)]
pub enum NotFound {
    Default(TransactionData),
}

impl NotFound {
    pub fn next(&self, request: crate::Request) -> Result<Response, String> {
        match self {
            _ => Ok(create_final_response_from(request)),
        }
    }
}

fn create_final_response_from(request: crate::Request) -> Response {
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
    let cseq = request.cseq().expect("request CallId header").clone();
    headers.push(Header::CSeq(cseq.0, cseq.1));
    headers.push(Header::ContentLength(0));
    headers.push(Header::Server("viska".into()));

    Response {
        code: 404,
        version: Default::default(),
        headers: headers,
        body: vec![],
    }
}

impl From<store::Transaction> for NotFound {
    fn from(record: store::Transaction) -> Self {
        match record.state {
            store::TransactionState::Trying => Self::Default(TransactionData {
                branch_id: record.branch_id,
                dialog_id: record.dialog_id,
            }),
            store::TransactionState::Proceeding => Self::Default(TransactionData {
                branch_id: record.branch_id,
                dialog_id: record.dialog_id,
            }),
            store::TransactionState::Completed => Self::Default(TransactionData {
                branch_id: record.branch_id,
                dialog_id: record.dialog_id,
            }),
            store::TransactionState::Terminated => Self::Default(TransactionData {
                branch_id: record.branch_id,
                dialog_id: record.dialog_id,
            }),
        }
    }
}

impl Into<store::DirtyTransaction> for NotFound {
    fn into(self) -> store::DirtyTransaction {
        match self {
            NotFound::Default(data) => store::DirtyTransaction {
                state: Some(store::TransactionState::Trying),
                branch_id: Some(data.branch_id),
                dialog_id: Some(data.dialog_id),
                ..Default::default()
            },
        }
    }
}
