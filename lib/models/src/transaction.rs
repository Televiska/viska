//use common::libsip::headers::Headers;
use crate::Response;
use common::uuid::Uuid;

#[derive(Debug, Clone)]
pub struct TransactionData {
    pub branch_id: String,
    pub dialog_id: i64,
}

#[derive(Debug, Clone)]
pub enum Transaction {
    Trying(TransactionData),
    Proceeding(TransactionData),
    Completed(TransactionData),
    Terminated(TransactionData),
}

impl Transaction {
    pub fn next(&self, request: crate::Request) -> Result<Response, String> {
        match self {
            Transaction::Trying(_data) => Ok(create_final_response_from(request)),
            _ => Err("wrong transaction state".into()),
        }
    }
}

fn create_final_response_from(request: crate::Request) -> Response {
    use common::libsip::{
        headers::{Header, Headers},
        uri::{Domain, UriParam},
    };
    use std::net::Ipv4Addr;

    let mut headers = Headers::new();
    let mut via_header = request.via_header().expect("request Via header").clone();
    let uri = via_header.uri.clone();
    let uri = uri.parameters(vec![
        UriParam::RPort(Some(5066)),
        UriParam::Branch(request.via_header_branch().expect("the branch").clone()),
        UriParam::Received(Domain::Ipv4(Ipv4Addr::new(192, 168, 1, 223), None)),
    ]);
    via_header.uri = uri;
    headers.push(Header::Via(via_header));
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
    let mut contact = request
        .contact_header()
        .expect("request contact header")
        .clone();
    contact.set_param("expires", Some("600"));
    headers.push(Header::Contact(contact));
    headers.push(Header::ContentLength(0));
    headers.push(Header::Server("viska".into()));

    Response {
        code: 200,
        version: Default::default(),
        headers,
        body: vec![],
    }
}

impl From<store::Transaction> for Transaction {
    fn from(record: store::Transaction) -> Self {
        match record.state {
            store::TransactionState::Trying => Self::Trying(TransactionData {
                branch_id: record.branch_id,
                dialog_id: record.dialog_id,
            }),
            store::TransactionState::Proceeding => Self::Proceeding(TransactionData {
                branch_id: record.branch_id,
                dialog_id: record.dialog_id,
            }),
            store::TransactionState::Completed => Self::Completed(TransactionData {
                branch_id: record.branch_id,
                dialog_id: record.dialog_id,
            }),
            store::TransactionState::Terminated => Self::Terminated(TransactionData {
                branch_id: record.branch_id,
                dialog_id: record.dialog_id,
            }),
        }
    }
}

impl Into<store::DirtyTransaction> for Transaction {
    fn into(self) -> store::DirtyTransaction {
        match self {
            Transaction::Trying(data) => store::DirtyTransaction {
                state: Some(store::TransactionState::Trying),
                branch_id: Some(data.branch_id),
                dialog_id: Some(data.dialog_id),
                ..Default::default()
            },
            Transaction::Proceeding(data) => store::DirtyTransaction {
                state: Some(store::TransactionState::Proceeding),
                branch_id: Some(data.branch_id),
                dialog_id: Some(data.dialog_id),
                ..Default::default()
            },
            Transaction::Completed(data) => store::DirtyTransaction {
                state: Some(store::TransactionState::Completed),
                branch_id: Some(data.branch_id),
                dialog_id: Some(data.dialog_id),
                ..Default::default()
            },
            Transaction::Terminated(data) => store::DirtyTransaction {
                state: Some(store::TransactionState::Completed),
                branch_id: Some(data.branch_id),
                dialog_id: Some(data.dialog_id),
                ..Default::default()
            },
        }
    }
}
