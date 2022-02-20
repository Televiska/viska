mod states;

pub use states::{Confirmed, Deleted, Early, Errored, Unconfirmed};

use crate::Error;
use common::rsip::{self, prelude::*};
use common::tokio::time::Instant;
use models::{Handlers, transport::RequestMsg};

#[derive(Debug)]
pub struct DialogSm {
    pub id: Option<String>,
    pub call_id: rsip::headers::CallId,
    pub transaction_id: String,
    pub local_tag: String,
    pub local_seqn: u32,
    pub local_uri: rsip::Uri,
    pub remote_tag: Option<String>,
    pub remote_seqn: Option<u32>,
    pub remote_uri: rsip::Uri,
    pub remote_target: Option<rsip::Uri>,
    pub msg: RequestMsg,
    pub state: DialogState,
    pub created_at: Instant,
    pub handlers: Handlers,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum DialogState {
    Unconfirmed(Unconfirmed),
    Early(Early),
    Confirmed(Confirmed),
    Deleted(Deleted),
    Errored(Errored),
}

impl std::fmt::Display for DialogState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unconfirmed(_) => write!(f, "DialogState::Unconfirmed"),
            Self::Early(_) => write!(f, "DialogState::Early"),
            Self::Confirmed(_) => write!(f, "DialogState::Confirmed"),
            Self::Deleted(_) => write!(f, "DialogState::Deleted"),
            Self::Errored(_) => write!(f, "DialogState::Errored"),
        }
    }
}

#[allow(dead_code)]
impl DialogSm {
    pub fn new(handlers: Handlers, msg: RequestMsg) -> Result<Self, Error> {
        Ok(Self {
            id: None,
            call_id: msg.sip_request.call_id_header()?.clone(),
            transaction_id: msg.sip_request.transaction_id()?.expect("transaction id").into(),
            local_tag: msg
                .sip_request
                .from_header()?
                .typed()?
                .tag()
                .ok_or(Error::from("missing from tag"))?
                .clone()
                .into(),
            local_seqn: msg.sip_request.cseq_header()?.typed()?.seq as u32,
            local_uri: msg.sip_request.from_header()?.typed()?.uri.clone(),
            remote_tag: None,
            remote_seqn: None,
            remote_uri: msg.sip_request.to_header()?.typed()?.uri.clone(),
            remote_target: None,
            msg,
            state: DialogState::Unconfirmed(Default::default()),
            created_at: Instant::now(),
            handlers,
        })
    }
}

#[allow(dead_code)]
fn compute_dialog_id(
    call_id: rsip::headers::CallId,
    local_tag: String,
    remote_tag: String,
) -> String {
    format!(
        "{}-{}-{}",
        Into::<String>::into(call_id),
        local_tag,
        remote_tag
    )
}
