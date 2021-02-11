mod states;

pub use states::{Accepted, Calling, Completed, Errored, Proceeding, Terminated};

use crate::Error;
use crate::SipManager;
use common::{
    rsip::{self, prelude::*},
    tokio::time::Instant,
};
use models::{
    transport::{RequestMsg, ResponseMsg},
    RequestExt,
};
use std::sync::Arc;

//should come from config
pub static TIMER_T1: u64 = 500;
pub static TIMER_B: u64 = 64 * TIMER_T1;
pub static TIMER_M: u64 = 64 * TIMER_T1;
pub static TIMER_D: u64 = 32000;

#[allow(dead_code)]
#[derive(Debug)]
pub struct TrxStateMachine {
    pub id: String,
    pub state: TrxState,
    pub msg: RequestMsg,
    pub created_at: Instant,
    sip_manager: Arc<SipManager>,
}

#[derive(Debug)]
pub enum TrxState {
    Calling(Calling),
    Proceeding(Proceeding),
    Completed(Completed),
    Accepted(Accepted),
    Terminated(Terminated),
    Errored(Errored),
}

impl std::fmt::Display for TrxState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Calling(_) => write!(f, "TrxState::Calling"),
            Self::Proceeding(_) => write!(f, "TrxState::Proceeding"),
            Self::Completed(_) => write!(f, "TrxState::Completed"),
            Self::Accepted(_) => write!(f, "TrxState::Accepted"),
            Self::Terminated(_) => write!(f, "TrxState::Terminated"),
            Self::Errored(_) => write!(f, "TrxState::Errored"),
        }
    }
}

impl TrxStateMachine {
    pub fn new(sip_manager: Arc<SipManager>, msg: RequestMsg) -> Result<Self, Error> {
        Ok(Self {
            id: msg.sip_request.transaction_id()?,
            state: TrxState::Calling(Default::default()),
            msg,
            created_at: Instant::now(),
            sip_manager,
        })
    }

    pub async fn next(&mut self, response: Option<rsip::Response>) {
        let result = match response {
            Some(response) => self.next_step_with(response).await,
            None => self.next_step().await,
        };

        match result {
            Ok(()) => (),
            Err(error) => self.error(
                format!("transaction {} errored: {}", self.id, error.to_string()),
                None,
            ),
        };
    }

    pub fn is_active(&self) -> bool {
        !matches!(self.state, TrxState::Errored(_) | TrxState::Terminated(_))
    }

    async fn next_step(&mut self) -> Result<(), Error> {
        match &self.state {
            TrxState::Calling(calling) => {
                match (calling.has_timedout(), calling.should_retransmit()) {
                    (true, _) => self.terminate(),
                    (false, true) => {
                        self.sip_manager
                            .transport
                            .send(self.msg.clone().into())
                            .await?;
                        self.state = TrxState::Calling(calling.retransmit());
                    }
                    (false, false) => (),
                }
            }
            TrxState::Completed(completed) => {
                if completed.should_terminate() {
                    self.terminate();
                }
            }
            TrxState::Accepted(accepted) => {
                if accepted.should_terminate() {
                    self.terminate();
                }
            }
            _ => (),
        };

        Ok(())
    }

    async fn next_step_with(&mut self, response: rsip::Response) -> Result<(), Error> {
        use rsip::common::StatusCodeKind;

        match (&self.state, response.status_code.kind()) {
            (TrxState::Calling(_), StatusCodeKind::Provisional) => {
                self.sip_manager
                    .core
                    .process_incoming_message(self.response_msg_from(response.clone()).into())
                    .await;
                self.proceed(response);
            }
            (TrxState::Calling(_), StatusCodeKind::Successful) => {
                self.sip_manager
                    .core
                    .process_incoming_message(self.response_msg_from(response.clone()).into())
                    .await;
                self.accept(response);
            }
            (TrxState::Calling(_), _) => {
                self.sip_manager
                    .core
                    .process_incoming_message(self.response_msg_from(response.clone()).into())
                    .await;
                self.complete(response);
            }
            (TrxState::Proceeding(_), StatusCodeKind::Provisional) => {
                self.sip_manager
                    .core
                    .process_incoming_message(self.response_msg_from(response.clone()).into())
                    .await;
                self.update_response(response);
            }
            (TrxState::Proceeding(_), StatusCodeKind::Successful) => {
                self.sip_manager
                    .core
                    .process_incoming_message(self.response_msg_from(response.clone()).into())
                    .await;
                self.accept(response);
            }
            (TrxState::Proceeding(_), _) => {
                self.sip_manager
                    .core
                    .process_incoming_message(self.response_msg_from(response.clone()).into())
                    .await;
                self.send_ack_request_from(response.clone()).await?;
                self.complete(response);
            }
            (TrxState::Accepted(_), StatusCodeKind::Successful) => {
                self.sip_manager
                    .core
                    .process_incoming_message(self.response_msg_from(response.clone()).into())
                    .await;
                self.update_response(response);
            }
            (TrxState::Completed(_), kind) if kind > StatusCodeKind::Successful => {
                self.complete(response.clone());
                self.send_ack_request_from(response.clone()).await?;
                self.complete(response);
            }
            (_, _) => {
                self.error(
                    format!(
                        "unknown match: {}, {} for transaction {}",
                        response.status_code, self.state, self.id
                    ),
                    Some(response),
                );
            }
        };

        Ok(())
    }

    fn proceed(&mut self, response: rsip::Response) {
        self.state = TrxState::Proceeding(Proceeding {
            response,
            entered_at: Instant::now(),
        });
    }

    fn update_response(&mut self, response: rsip::Response) {
        match &self.state {
            TrxState::Proceeding(state) => {
                self.state = TrxState::Proceeding(Proceeding {
                    response,
                    entered_at: state.entered_at,
                })
            }
            TrxState::Completed(state) => {
                self.state = TrxState::Completed(Completed {
                    response,
                    entered_at: state.entered_at,
                })
            }
            TrxState::Accepted(state) => {
                self.state = TrxState::Accepted(Accepted {
                    response,
                    entered_at: state.entered_at,
                })
            }
            TrxState::Errored(state) => {
                self.state = TrxState::Errored(Errored {
                    response: Some(response),
                    entered_at: state.entered_at,
                    error: state.error.clone(),
                })
            }
            _ => self.error(
                format!("Asking to update response when state is {}", self.state),
                Some(response),
            ),
        };
    }

    fn complete(&mut self, response: rsip::Response) {
        self.state = TrxState::Completed(Completed {
            response,
            entered_at: Instant::now(),
        });
    }

    fn accept(&mut self, response: rsip::Response) {
        self.state = TrxState::Accepted(Accepted {
            response,
            entered_at: Instant::now(),
        });
    }

    fn terminate(&mut self) {
        let response: Option<rsip::Response> = match &self.state {
            TrxState::Accepted(accepted) => Some(accepted.clone().response),
            TrxState::Completed(completed) => Some(completed.clone().response),
            _ => None,
        };

        self.state = TrxState::Terminated(Terminated {
            response,
            entered_at: Instant::now(),
        });
    }

    fn error(&mut self, error: String, response: Option<rsip::Response>) {
        self.state = TrxState::Errored(Errored {
            entered_at: Instant::now(),
            response,
            error,
        });
    }

    fn response_msg_from(&self, response: rsip::Response) -> ResponseMsg {
        ResponseMsg {
            sip_response: response,
            peer: self.msg.peer,
            transport: self.msg.transport,
        }
    }

    fn request_msg_from(&self, request: rsip::Request) -> RequestMsg {
        RequestMsg {
            sip_request: request,
            peer: self.msg.peer,
            transport: self.msg.transport,
        }
    }

    async fn send_ack_request_from(&self, response: rsip::Response) -> Result<(), Error> {
        Ok(self
            .sip_manager
            .transport
            .send(
                self.request_msg_from(self.msg.sip_request.ack_request_with(response))
                    .into(),
            )
            .await?)
    }
}
