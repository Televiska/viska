use crate::common::{
    self,
    factories::prelude::*,
    snitches::{UaSnitch, TransportSnitch},
};
use ::common::ipnetwork::IpNetwork;
use ::common::rsip::{self, prelude::*};
use models::transport::RequestMsg;
use sip_server::{
    tu::impls::{UserAgent, Registrar},
    ReqProcessor, SipBuilder, SipManager, Transaction, TuLayer,
};
use std::sync::Arc;

async fn setup() -> (Registrar, Arc<SipManager>) {
    let sip_manager = SipBuilder::new::<UaSnitch, Transaction, TransportSnitch>()
        .expect("sip manager failed")
        .manager;

    let registrar = Registrar::new(Arc::downgrade(&sip_manager));

    (registrar, sip_manager)
}

#[tokio::test]
#[serial_test::serial]
async fn with_no_records_returns_empty_list() {
    let _ = common::setup();
    let (registrar, sip_manager) = setup().await;
    let transport = sip_manager.transport.clone();
    let transport = as_any!(transport, TransportSnitch);

    let res = registrar
        .process_incoming_request(RequestMsg {
            sip_request: requests::register_query_request(),
            ..Randomized::default()
        })
        .await;
    assert!(res.is_ok(), "returns: {:?}", res);
    assert_eq!(transport.messages.len().await, 1);
    assert_eq!(
        transport.messages.first_response().await.status_code,
        200.into()
    );
    assert!(transport
        .messages
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
    let _ = common::setup();
    create_registration();
    create_registration();
    let (registrar, sip_manager) = setup().await;
    let transport = sip_manager.transport.clone();
    let transport = as_any!(transport, TransportSnitch);

    let res = registrar
        .process_incoming_request(RequestMsg {
            sip_request: requests::register_query_request(),
            ..Randomized::default()
        })
        .await;
    assert!(res.is_ok(), "returns: {:?}", res);
    assert_eq!(transport.messages.len().await, 1);
    assert_eq!(
        transport.messages.first_response().await.status_code,
        200.into()
    );
    assert_eq!(
        transport
            .messages
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
    let _ = common::setup();
    create_registration();
    let (registrar, sip_manager) = setup().await;
    let transport = sip_manager.transport.clone();
    let transport = as_any!(transport, TransportSnitch);

    let res = registrar
        .process_incoming_request(RequestMsg {
            sip_request: requests::register_request(),
            ..Randomized::default()
        })
        .await;
    assert!(res.is_ok(), "returns: {:?}", res);
    assert_eq!(transport.messages.len().await, 1);
    assert_eq!(
        transport.messages.first_response().await.status_code,
        200.into()
    );
    assert_eq!(
        transport
            .messages
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

    let _ = common::setup();
    let (registrar, sip_manager) = setup().await;
    let transport = sip_manager.transport.clone();
    let transport = as_any!(transport, TransportSnitch);
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
    assert!(res.is_err(), "returns: {:?}", res);
    assert_eq!(transport.messages.len().await, 0);
}

#[tokio::test]
#[serial_test::serial]
async fn delete_registration() {
    let _ = common::setup();

    let (registration, uri) = create_registration();
    let (registrar, sip_manager) = setup().await;
    let transport = sip_manager.transport.clone();
    let transport = as_any!(transport, TransportSnitch);

    let res = registrar
        .process_incoming_request(RequestMsg {
            sip_request: requests::register_delete_request_with_uri(uri),
            ..Randomized::default()
        })
        .await;
    assert!(res.is_ok(), "returns: {:?}", res);
    assert_eq!(transport.messages.len().await, 1);
    assert_eq!(
        transport.messages.first_response().await.status_code,
        200.into()
    );
    assert_eq!(
        transport
            .messages
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
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

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

    let foo: rsip::headers::Contact = new_registration
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
