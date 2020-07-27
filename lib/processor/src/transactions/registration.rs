use common::uuid::Uuid;
use std::net::Ipv4Addr;

impl super::TransactionFSM for models::transactions::Registration {
    fn next(&self, request: models::Request) -> Result<models::Response, String> {
        use models::transactions::Registration;

        match self {
            Registration::Trying(_data) => {
                update_registration_for(request.clone())?;
                Ok(create_final_response_from(request))
            }
            _ => Err("wrong transaction state".into()),
        }
    }
}

fn update_registration_for(request: models::Request) -> Result<models::Registration, String> {
    use std::convert::TryInto;

    Ok(
        store::Registration::upsert(TryInto::<models::UpdateRegistration>::try_into(request)?)
            .map_err(|e| e.to_string())?
            .into(),
    )
}

fn create_final_response_from(request: models::Request) -> models::Response {
    use common::libsip::{
        headers::{Header, Headers},
        uri::{Domain, UriParam},
    };

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

    models::Response {
        code: 200,
        version: Default::default(),
        headers,
        body: vec![],
    }
}
