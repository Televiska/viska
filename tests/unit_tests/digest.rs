use sip_helpers::auth::*;

#[tokio::test]
async fn generate_digest() {
    let mut header = AuthorizationHeader {
        username: "vasilakisfil".into(),
        realm: "192.168.1.223".into(),
        nonce: "ea9c8e88df84f1cec4341ae6cbe5a359".into(),
        uri: "sip:192.168.1.223".into(),
        algorithm: Algorithm::default(),
        cnonce: Some("b53d4995-0044-446c-964a-ad6342803a19".into()),
        qop: Some(Qop::Auth),
        nc: Some(1),
        opaque: None,
        response: None
    };

    header.with_digest_for("123123123".into());

    assert_eq!(header.response, Some("c33791267630ae12f36237d6526aae7f".to_string()));
    assert_eq!(header.verify_for("123123123".into()).expect("verify"), true);
}
