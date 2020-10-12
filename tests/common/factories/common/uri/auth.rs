use common::libsip::{self};

#[derive(Debug, Clone)]
pub struct Auth {
    pub username: String,
    pub password: Option<String>,
}

impl Default for Auth {
    fn default() -> Self {
        Self {
            username: "foo@example.com".into(),
            password: Some("123123123".into()),
        }
    }
}

impl Into<libsip::uri::UriAuth> for Auth {
    fn into(self) -> libsip::uri::UriAuth {
        libsip::uri::UriAuth {
            username: self.username,
            password: self.password,
        }
    }
}

impl From<(String, Option<String>)> for Auth {
    fn from(tuple: (String, Option<String>)) -> Self {
        Self {
            username: tuple.0,
            password: tuple.1,
        }
    }
}
