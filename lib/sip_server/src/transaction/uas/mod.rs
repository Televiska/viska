mod states;

pub use states::{Accepted, Completed, Confirmed, Errored, Proceeding, Terminated};

use crate::Error;
use crate::SipManager;
use common::{
    rsip::{self, prelude::*},
    tokio::time::Instant,
};
use models::transport::{RequestMsg, ResponseMsg};
use std::sync::Arc;

static TIMED_OUT: bool = true;
static DID_NOT_TIME_OUT: bool = false;

//should come from config
pub static TIMER_T1: u64 = 500;
pub static TIMER_T2: u64 = 4000;
pub static TIMER_G: u64 = TIMER_T1;
pub static TIMER_H: u64 = 64 * TIMER_T1;
pub static TIMER_T4: u64 = 5000;
pub static TIMER_I: u64 = TIMER_T4;
pub static TIMER_L: u64 = 64 * TIMER_T1;

#[allow(dead_code)]
#[derive(Debug)]
pub struct TrxStateMachine {
    pub id: String,
    pub state: TrxState,
    pub msg: RequestMsg,
    //uas (final) response, uas in this case is us
    pub response: rsip::Response,
    pub created_at: Instant,
    sip_manager: Arc<SipManager>,
}

#[derive(Debug)]
pub enum TrxState {
    Proceeding(Proceeding),
    Completed(Completed),
    Accepted(Accepted),
    Confirmed(Confirmed),
    Terminated(Terminated),
    Errored(Errored),
}

impl std::fmt::Display for TrxState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Proceeding(_) => write!(f, "TrxState::Proceeding"),
            Self::Completed(_) => write!(f, "TrxState::Completed"),
            Self::Accepted(_) => write!(f, "TrxState::Accepted"),
            Self::Confirmed(_) => write!(f, "TrxState::Confirmed"),
            Self::Terminated(_) => write!(f, "TrxState::Terminated"),
            Self::Errored(_) => write!(f, "TrxState::Errored"),
        }
    }
}

impl TrxStateMachine {
    pub fn new(
        sip_manager: Arc<SipManager>,
        msg: RequestMsg,
        response: Option<rsip::Response>,
    ) -> Result<Self, Error> {
        use models::RequestExt;

        let RequestMsg {
            sip_request,
            peer,
            transport,
        } = msg;

        Ok(Self {
            id: sip_request.transaction_id()?.expect("transaction_id").into(),
            state: TrxState::Proceeding(Default::default()),
            response: response.unwrap_or_else(|| sip_request.provisional_of(100)),
            msg: RequestMsg {
                sip_request,
                peer,
                transport,
            },
            created_at: Instant::now(),
            sip_manager,
        })
    }

    pub fn is_active(&self) -> bool {
        !matches!(self.state, TrxState::Errored(_) | TrxState::Terminated(_))
    }

    pub async fn next(&mut self, sip_message: Option<rsip::SipMessage>) -> Result<(), Error> {
        use rsip::SipMessage;
        let result = match sip_message {
            Some(SipMessage::Request(request)) => {
                self.next_step_with_incoming_request(request).await
            }
            Some(SipMessage::Response(response)) => {
                self.next_step_with_outgoing_response(response).await
            }
            None => self.next_step().await,
        };

        match result {
            Ok(()) => Ok(()),
            Err(error) => {
                let error_str = format!("transaction {} errored: {}", self.id, error.to_string());
                self.error(error_str.clone(), None);

                Err(error_str.into())
            }
        }
    }

    async fn next_step(&mut self) -> Result<(), Error> {
        match &self.state {
            TrxState::Completed(completed) => {
                match (completed.has_timedout(), completed.should_retransmit()) {
                    (true, _) => self.terminate(TIMED_OUT),
                    (false, true) => {
                        self.sip_manager
                            .transport
                            .send(self.response_msg_from(self.response.clone()).into())
                            .await?;
                        self.state = TrxState::Completed(completed.retransmit());
                    }
                    (false, false) => (),
                }
            }
            TrxState::Accepted(accepted) => {
                if accepted.should_terminate() {
                    self.terminate(TIMED_OUT);
                }
            }
            TrxState::Confirmed(confirmed) => {
                if confirmed.should_terminate() {
                    self.terminate(DID_NOT_TIME_OUT);
                }
            }
            _ => (),
        };

        Ok(())
    }

    //TODO: how do we make sure that the request is the same as the original ?
    //we need to implement that on transaction id, making sure whatever enters here is correct
    //we should take into account the TU response as well
    //TODO: here we assume is the same as the original or an ACK
    async fn next_step_with_incoming_request(
        &mut self,
        request: rsip::Request,
    ) -> Result<(), Error> {
        use rsip::common::Method;

        match (&self.state, &request.method) {
            (TrxState::Proceeding(_), Method::Invite) => {
                self.sip_manager
                    .transport
                    .send(self.response_msg_from(self.response.clone()).into())
                    .await?;
            }
            (TrxState::Completed(_), Method::Invite) => {
                self.sip_manager
                    .transport
                    .send(self.response_msg_from(self.response.clone()).into())
                    .await?;
            }
            (TrxState::Completed(_), Method::Ack) => {
                self.confirm(request);
            }
            (TrxState::Accepted(_), Method::Invite) => {
                self.sip_manager
                    .transport
                    .send(self.response_msg_from(self.response.clone()).into())
                    .await?;
            }
            (TrxState::Accepted(_), Method::Ack) => {
                self.sip_manager
                    .tu
                    .process_incoming_message(self.request_msg_from(request).into())
                    .await;
            }
            (TrxState::Confirmed(_), Method::Ack) => {
                //absorb ack
            }
            _ => self.error(
                format!(
                    "unknown transition for {} and {}",
                    self.state, request.method
                ),
                Some(request.into()),
            ),
        }

        Ok(())
    }

    async fn next_step_with_outgoing_response(
        &mut self,
        response: rsip::Response,
    ) -> Result<(), Error> {
        use rsip::common::StatusCodeKind;

        match (&self.state, response.status_code.kind()) {
            (TrxState::Proceeding(_), StatusCodeKind::Provisional) => {
                self.response = response.clone();
                self.sip_manager
                    .transport
                    .send(self.response_msg_from(response).into())
                    .await?;
            }
            (TrxState::Proceeding(_), StatusCodeKind::Successful) => {
                self.response = response.clone();
                self.sip_manager
                    .transport
                    .send(self.response_msg_from(response).into())
                    .await?;
                self.accept();
            }
            (TrxState::Proceeding(_), _) => {
                self.response = response.clone();
                self.sip_manager
                    .transport
                    .send(self.response_msg_from(response).into())
                    .await?;
                self.complete();
            }
            (TrxState::Accepted(_), StatusCodeKind::Successful) => {
                self.sip_manager
                    .transport
                    .send(self.response_msg_from(self.response.clone()).into())
                    .await?;
            }
            _ => self.error(
                format!(
                    "unknown transition for {} and {}",
                    self.state, response.status_code
                ),
                Some(response.into()),
            ),
        }

        Ok(())
    }

    fn complete(&mut self) {
        self.state = TrxState::Completed(Default::default());
    }

    fn accept(&mut self) {
        self.state = TrxState::Accepted(Default::default());
    }

    fn confirm(&mut self, request: rsip::Request) {
        self.state = TrxState::Confirmed(Confirmed {
            request,
            entered_at: Instant::now(),
        });
    }

    fn terminate(&mut self, timedout: bool) {
        self.state = TrxState::Terminated(Terminated {
            timedout,
            entered_at: Instant::now(),
        });
    }

    fn error(&mut self, error: String, sip_message: Option<rsip::SipMessage>) {
        self.state = TrxState::Errored(Errored {
            entered_at: Instant::now(),
            sip_message,
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
}
