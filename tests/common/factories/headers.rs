use common::libsip::{self, *};

pub fn via() -> Header {
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
    let uri = Uri::sip(domain!("example.com")).auth(uri_auth!("guy"));
    Header::From(named_header!(uri, "Guy"))
}

pub fn to() -> Header {
    let uri = Uri::sip(domain!("example.com")).auth(uri_auth!("guy"));
    Header::To(named_header!(uri, "Guy"))
}

pub fn call_id() -> Header {
    Header::CallId("Sofngfwertwowert.0".into())
}

pub fn cseq(method: Method) -> Header {
    Header::CSeq(444, method)
}

pub fn max_forwards(num: Option<u32>) -> Header {
    Header::MaxForwards(num.unwrap_or(70))
}

pub fn contact() -> Header {
    let uri = Uri::sip(domain!("example.com")).auth(uri_auth!("guy"));
    Header::Contact(contact_header!(uri, "Guy"))
}

pub fn contact_length(length: Option<u32>) -> Header {
    Header::ContentLength(length.unwrap_or(0))
}

pub fn user_agent() -> Header {
    Header::UserAgent("viskaphone".into())
}
