pub trait SipMessageExt {
    fn dialog_id(&self) -> Option<String>;
}

impl SipMessageExt for rsip::Request {
    pub fn dialog_id(&self) -> Option<String> {
        match (self.call_id(), self.from_header_tag(), self.to_header_tag()) {
            (Ok(call_id), Ok(from_tag), Ok(to_tag)) => {
                Some(format!("{}-{}-{}", call_id, from_tag, to_tag))
            }
            _ => None,
        }
    }
}
