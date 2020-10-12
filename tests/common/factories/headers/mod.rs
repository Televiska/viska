pub mod call_id;
pub mod contact;
pub mod content_length;
pub mod cseq;
pub mod from;
pub mod max_forwards;
pub mod named;
pub mod to;
pub mod user_agent;
pub mod via;

pub use call_id::CallId;
pub use contact::Contact;
pub use content_length::ContentLength;
pub use cseq::CSeq;
pub use from::From;
pub use max_forwards::MaxForwards;
pub use named::{ContactParam, NamedHeader, NamedParam};
pub use to::To;
pub use user_agent::UserAgent;
pub use via::Via;

use common::libsip::{headers::Header, Method};

pub fn via() -> Header {
    use common::libsip::{self, *};

    let header = ViaHeader {
        version: Version::default(),
        transport: Transport::Udp,
        uri: Uri::new_schemaless(domain!("example.com"))
            .parameter(UriParam::RPort(None))
            .parameter(UriParam::Branch("z9hG4bK7Q6y313Qrt6Uc".into())),
    };

    Header::Via(header)
}

pub fn from() -> Header {
    use common::libsip::{self, *};

    let uri = Uri::sip(domain!("example.com")).auth(uri_auth!("guy"));
    Header::From(named_header!(uri, "Guy"))
}

pub fn to() -> Header {
    use common::libsip::{self, *};

    let uri = Uri::sip(domain!("example.com")).auth(uri_auth!("guy"));
    Header::To(named_header!(uri, "Guy"))
}

pub fn call_id() -> Header {
    use common::libsip::{self, *};

    Header::CallId("Sofngfwertwowert.0".into())
}

pub fn cseq(method: Method) -> Header {
    use common::libsip::{self, *};

    Header::CSeq(444, method)
}

pub fn max_forwards(num: Option<u32>) -> Header {
    use common::libsip::{self, *};

    Header::MaxForwards(num.unwrap_or(70))
}

pub fn contact() -> Header {
    use common::libsip::{self, *};

    let uri = Uri::sip(domain!("example.com")).auth(uri_auth!("guy"));
    Header::Contact(contact_header!(uri, "Guy"))
}

pub fn contact_length(length: Option<u32>) -> Header {
    use common::libsip::{self, *};

    Header::ContentLength(length.unwrap_or(0))
}

pub fn user_agent() -> Header {
    use common::libsip::{self, *};

    Header::UserAgent("viskaphone".into())
}
