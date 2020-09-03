use crate::auth::{Algorithm, Qop};
use common::{
    libsip::headers::{AuthHeader, AuthSchema},
};
use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub struct WwwAuthenticateHeader {
    //that should be vec
    pub domain: Option<String>,
    pub realm: String,
    pub nonce: String,
    pub opaque: Option<String>,
    pub stale: bool,
    pub algorithm: Algorithm,
    //latest rfc requires qop
    pub qop: Option<Qop>,
    //pub userhash: bool,
    //pub charset: Charset,
}

impl WwwAuthenticateHeader {
    pub fn new(realm: String, nonce: String) -> Self {
        Self {
            domain: Some(realm.clone()),
            realm,
            nonce,
            opaque: None,
            stale: false,
            algorithm: Algorithm::default(),
            qop: Some(Qop::Auth),
        }
    }
}


impl Into<AuthHeader> for WwwAuthenticateHeader {
    fn into(self) -> AuthHeader {
        let mut map: HashMap<String, String> = HashMap::new();
        match self.domain {
            Some(domain) => {
                map.insert("domain".into(), domain);
            }
            _ => (),
        };
        map.insert("realm".into(), self.realm);
        map.insert("nonce".into(), self.nonce);
        match self.opaque {
            Some(opaque) => {
                map.insert("opaque".into(), opaque);
            }
            _ => (),
        };
        match self.stale {
            true => map.insert("stale".into(), "TRUE".into()),
            false => map.insert("stale".into(), "FALSE".into()),
        };

        map.insert("algorithm".into(), self.algorithm.into());
        match self.qop {
            Some(qop) => {
                map.insert("qop".into(), qop.into());
            }
            None => (),
        };

        AuthHeader(AuthSchema::Digest, map)
    }
}
