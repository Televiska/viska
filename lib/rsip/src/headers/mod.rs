mod header;
pub use header::named;
pub use header::Header;
pub use header::*;

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Headers(Vec<Header>);

impl Headers {
    pub fn push(&mut self, h: Header) {
        self.0.push(h)
    }

    pub fn unique_push(&mut self, h: Header) {
        self.0
            .retain(|s| std::mem::discriminant(s) != std::mem::discriminant(&h));
        self.push(h);
    }

    pub fn iter(&self) -> impl Iterator<Item = &Header> {
        self.0.iter()
    }

    pub fn extend(&mut self, i: Vec<Header>) {
        self.0.extend(i)
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Header> {
        self.0.iter_mut()
    }
}

impl IntoIterator for Headers {
    type IntoIter = ::std::vec::IntoIter<Self::Item>;
    type Item = Header;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl std::convert::From<Header> for Headers {
    fn from(header: Header) -> Self {
        Self(vec![header])
    }
}

impl std::convert::From<Vec<Header>> for Headers {
    fn from(headers: Vec<Header>) -> Self {
        Self(headers)
    }
}

impl Into<Vec<Header>> for Headers {
    fn into(self) -> Vec<Header> {
        self.0
    }
}

impl std::fmt::Display for Headers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use libsip::core::message::display_headers_and_body;
        display_headers_and_body(f, &Into::<libsip::Headers>::into(self.clone()), &[])
    }
}

impl Into<Vec<libsip::headers::Header>> for Headers {
    fn into(self) -> Vec<libsip::headers::Header> {
        self.0.into_iter().map(Into::into).collect::<_>()
    }
}

impl Into<libsip::headers::Headers> for Headers {
    fn into(self) -> libsip::headers::Headers {
        libsip::headers::Headers(self.into())
    }
}
