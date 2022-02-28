use crate::common::{factories::prelude::*, snitches::SpySnitch};
use common::{ipnetwork::IpNetwork, rsip::{self, headers::UntypedHeader}};
use models::{
    transaction::TransactionLayerMsg,
    transport::{RequestMsg, TransportLayerMsg},
    tu::TuLayerMsg,
};
use sip_server::{tu::elements::Registrar, ReqProcessor};

pub async fn setup() -> (
    SpySnitch<TuLayerMsg>,
    SpySnitch<TransactionLayerMsg>,
    SpySnitch<TransportLayerMsg>,
) {
    let (handlers, receivers) = models::channels_builder();
    let transport = SpySnitch::new(handlers.clone(), receivers.transport).expect("transport");
    let transaction = SpySnitch::new(handlers.clone(), receivers.transaction).expect("transaction");
    let tu = SpySnitch::new(handlers.clone(), receivers.tu).expect("tu");

    (tu, transaction, transport)
}

#[tokio::test]
#[serial_test::serial]
async fn with_no_records_returns_empty_list() {
    let _ = crate::common::setup();
    let (_, _, transport) = setup().await;

    let registrar = Registrar::new(transport.handlers());

    registrar
        .process_incoming_request(RequestMsg {
            sip_request: requests::register_query_request(),
            ..Randomized::default()
        })
        .await
        .unwrap();

    assert_eq!(transport.messages().await.len().await, 1);
    assert_eq!(
        transport
            .messages()
            .await
            .first_response()
            .await
            .status_code,
        200.into()
    );
    assert!(transport
        .messages()
        .await
        .first_response()
        .await
        .headers
        .iter()
        .find(|h| matches!(h, rsip::Header::Contact(_)))
        .is_none());
}

#[tokio::test]
#[serial_test::serial]
async fn with_records_returns_a_list_of_contacts() {
    let _ = crate::common::setup();
    let (_, _, transport) = setup().await;

    let registrar = Registrar::new(transport.handlers());

    create_registration();
    create_registration();

    registrar
        .process_incoming_request(RequestMsg {
            sip_request: requests::register_query_request(),
            ..Randomized::default()
        })
        .await
        .unwrap();
    assert_eq!(transport.messages().await.len().await, 1);
    assert_eq!(
        transport
            .messages()
            .await
            .first_response()
            .await
            .status_code,
        200.into()
    );
    assert_eq!(
        transport
            .messages()
            .await
            .first_response()
            .await
            .headers
            .iter()
            .filter(|h| matches!(h, rsip::Header::Contact(_)))
            .count(),
        2
    );
}

#[tokio::test]
#[serial_test::serial]
async fn with_new_register_request_saves_the_contact() {
    let _ = crate::common::setup();
    let (_, _, transport) = setup().await;

    let registrar = Registrar::new(transport.handlers());

    create_registration();

    registrar
        .process_incoming_request(RequestMsg {
            sip_request: requests::register_request(),
            ..Randomized::default()
        })
        .await
        .unwrap();
    assert_eq!(transport.messages().await.len().await, 1);
    assert_eq!(
        transport
            .messages()
            .await
            .first_response()
            .await
            .status_code,
        200.into()
    );
    assert_eq!(
        transport
            .messages()
            .await
            .first_response()
            .await
            .headers
            .iter()
            .filter(|h| matches!(h, rsip::Header::Contact(_)))
            .count(),
        2
    );

    assert_eq!(
        store::Registration::count(Default::default()).expect("registrations count"),
        2
    )
}

#[tokio::test]
#[serial_test::serial]
async fn with_wrong_from_to_register() {
    use rsip::Uri;

    let _ = crate::common::setup();
    let (_, _, transport) = setup().await;

    let registrar = Registrar::new(transport.handlers());

    let mut request = requests::register_request();
    request
        .headers
        .unique_push(rsip::typed::To::from(Uri::default().with_user("another")).into());

    let res = registrar
        .process_incoming_request(RequestMsg {
            sip_request: request,
            ..Randomized::default()
        })
        .await;
    assert!(res.is_err());
    assert_eq!(transport.messages().await.len().await, 0);
}

#[tokio::test]
#[serial_test::serial]
async fn delete_registration() {
    let _ = crate::common::setup();
    let (_, _, transport) = setup().await;

    let registrar = Registrar::new(transport.handlers());

    let (_registration, uri) = create_registration();

    registrar
        .process_incoming_request(RequestMsg {
            sip_request: requests::register_delete_request_with_uri(uri),
            ..Randomized::default()
        })
        .await
        .unwrap();
    assert_eq!(transport.messages().await.len().await, 1);
    assert_eq!(
        transport
            .messages()
            .await
            .first_response()
            .await
            .status_code,
        200.into()
    );
    assert_eq!(
        transport
            .messages()
            .await
            .first_response()
            .await
            .headers
            .iter()
            .filter(|h| matches!(h, rsip::Header::Contact(_)))
            .count(),
        0
    );

    assert_eq!(
        store::Registration::count(Default::default()).expect("registrations count"),
        0
    )
}

fn create_registration() -> (store::Registration, rsip::Uri) {
    use ::common::chrono::{Duration, Utc};
    use std::convert::TryInto;

    let ip_address: IpNetwork = IpAddrBuilder::localhost().into();
    let user: String = "filippos".into();

    let uri = rsip::Uri {
        scheme: Some(rsip::Scheme::default()),
        host_with_port: rsip::HostWithPort::from(ip_address.clone().ip()),
        auth: Some(rsip::Auth {
            user: user.clone(),
            password: None,
        }),
        params: vec![],
        headers: vec![].into(),
    };

    //TODO: should impl Randomized default
    let mut new_registration = store::DirtyRegistration {
        username: Some(user),
        domain: Some("localhost".into()),
        expires: Some(Utc::now() + Duration::minutes(100)),
        call_id: Some(rsip::headers::CallId::default().value().into()),
        cseq: Some(1),
        user_agent: Some(rsip::headers::UserAgent::default().value().into()),
        instance: None,
        ip_address: Some(ip_address),
        port: Some(5060),
        transport: Some(rsip::Transport::default().into()),
        contact: None,
        contact_uri: Some(uri.to_string()),
    };

    let contact_header: rsip::Header = rsip::headers::Contact::new(rsip::typed::Contact {
        display_name: None,
        uri: uri.clone(),
        params: Default::default(),
    })
    .into();

    new_registration.contact = Some(
        contact_header
            .to_string()
            .splitn(2, ':')
            .last()
            .expect("last")
            .to_owned(),
    );

    let _: rsip::headers::Contact = new_registration
        .contact
        .clone()
        .expect("contact")
        .try_into()
        .expect("contact try into");

    (
        store::Registration::create(new_registration).expect("registration create"),
        uri,
    )
}
