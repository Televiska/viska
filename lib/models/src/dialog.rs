use crate::transactions::{NotFound, Registration};

#[derive(Debug, Clone)]
pub struct Dialog {
    pub computed_id: String,
    pub call_id: String,
    pub from_tag: String,
    pub to_tag: String,
    pub flow: DialogFlow,
}

#[derive(Debug, Clone)]
pub enum DialogFlow {
    Registration(Registration),
    Invite(NotFound),
    Publish(NotFound),
}
