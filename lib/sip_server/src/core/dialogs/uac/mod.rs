mod states;

pub use states::{Confirmed, Deleted, Early, Errored, Unconfirmed};

use crate::Error;
use crate::SipManager;
use models::{transport::RequestMsg, SipMessageExt};
use rsip::{common::Uri, headers::CallId, message::HeadersExt};
use std::sync::Arc;
use tokio::time::Instant;

#[derive(Debug)]
pub struct DgStateMachine {
    pub id: Option<String>,
    pub call_id: CallId,
    pub transaction_id: String,
    pub local_tag: String,
    pub local_seqn: u32,
    pub local_uri: Uri,
    pub remote_tag: Option<String>,
    pub remote_seqn: Option<u32>,
    pub remote_uri: Uri,
    pub remote_target: Option<Uri>,
    pub msg: RequestMsg,
    pub state: DgState,
    pub created_at: Instant,
    pub sip_manager: Arc<SipManager>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum DgState {
    Unconfirmed(Unconfirmed),
    Early(Early),
    Confirmed(Confirmed),
    Deleted(Deleted),
    Errored(Errored),
}

impl std::fmt::Display for DgState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unconfirmed(_) => write!(f, "DgState::Unconfirmed"),
            Self::Early(_) => write!(f, "DgState::Early"),
            Self::Confirmed(_) => write!(f, "DgState::Confirmed"),
            Self::Deleted(_) => write!(f, "DgState::Deleted"),
            Self::Errored(_) => write!(f, "DgState::Errored"),
        }
    }
}

#[allow(dead_code)]
impl DgStateMachine {
    pub fn new(sip_manager: Arc<SipManager>, msg: RequestMsg) -> Result<Self, Error> {
        Ok(Self {
            id: None,
            call_id: msg.sip_request.call_id_header()?.clone(),
            transaction_id: msg.sip_request.transaction_id()?,
            local_tag: msg
                .sip_request
                .from_header()?
                .tag()
                .ok_or(Error::from("missing from tag"))?
                .clone()
                .into(),
            local_seqn: msg.sip_request.cseq_header()?.seq,
            local_uri: msg.sip_request.from_header()?.0.uri.clone(),
            remote_tag: None,
            remote_seqn: None,
            remote_uri: msg.sip_request.to_header()?.0.uri.clone(),
            remote_target: None,
            msg,
            state: DgState::Unconfirmed(Default::default()),
            created_at: Instant::now(),
            sip_manager,
        })
    }
}

fn compute_dialog_id(call_id: CallId, local_tag: String, remote_tag: String) -> String {
    format!("{}-{}-{}", Into::<String>::into(call_id), local_tag, remote_tag)
}
