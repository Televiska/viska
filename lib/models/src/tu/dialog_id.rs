use std::fmt::Display;

//TODO: impl UnconfirmedDialogId and DialogId using the AsRef trait
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct DialogId {
    call_id: String,
    local_tag: String,
    remote_tag: Option<String>,
}

impl DialogId {
    pub fn new(
        call_id: impl Display,
        local_tag: impl Display,
        remote_tag: Option<impl Display>,
    ) -> Self {
        Self {
            call_id: call_id.to_string(),
            local_tag: local_tag.to_string(),
            remote_tag: remote_tag.map(|s| s.to_string()),
        }
    }

    //TODO: should be easy to optimize with memoization
    pub fn prefixed(&self) -> Self {
        match self.remote_tag {
            Some(_) => {
                let mut cloned = self.clone();
                cloned.remote_tag = None;
                cloned
            }
            None => self.clone(),
        }
    }

    pub fn is_unconfirmed(&self) -> bool {
        self.remote_tag.is_none()
    }
}

impl Display for DialogId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.remote_tag {
            Some(remote_tag) => write!(f, "{}-{}-{}", self.call_id, self.local_tag, remote_tag),
            None => write!(f, "{}-{}", self.call_id, self.local_tag),
        }
    }
}
