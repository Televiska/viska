mod states;

pub use states::{Accepted, Completed, Confirmed, Proceeding, Terminated};

use crate::Error;
use common::{
    rsip::{self, prelude::*},
    tokio::time::Instant,
};
use models::{transaction::TransactionId, Handlers};

//TODO: add state checks as well for better guarantees, look at dialogs

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
    pub id: TransactionId,
    pub state: TrxState,
    pub request: rsip::Request,
    //uas (final) response, uas in this case is us
    pub response: rsip::Response,
    pub created_at: Instant,
    handlers: Handlers,
}

#[derive(Debug)]
pub enum TrxState {
    Proceeding(Proceeding),
    Completed(Completed),
    Accepted(Accepted),
    Confirmed(Confirmed),
    Terminated(Terminated),
}

impl std::fmt::Display for TrxState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Proceeding(_) => write!(f, "TrxState::Proceeding"),
            Self::Completed(_) => write!(f, "TrxState::Completed"),
            Self::Accepted(_) => write!(f, "TrxState::Accepted"),
            Self::Confirmed(_) => write!(f, "TrxState::Confirmed"),
            Self::Terminated(_) => write!(f, "TrxState::Terminated"),
        }
    }
}

impl TrxStateMachine {
    pub fn new(
        handlers: Handlers,
        request: rsip::Request,
        response: Option<rsip::Response>,
    ) -> Result<Self, Error> {
        use models::rsip_ext::*;

        Ok(Self {
            id: request.transaction_id()?.expect("transaction_id"),
            state: TrxState::Proceeding(Default::default()),
            response: response.unwrap_or_else(|| request.provisional_of(100)),
            request,
            created_at: Instant::now(),
            handlers,
        })
    }

    pub fn is_active(&self) -> bool {
        !matches!(self.state, TrxState::Terminated(_))
    }

    pub async fn next(&mut self, sip_message: Option<rsip::SipMessage>) {
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
            Ok(()) => (),
            Err(error) => self.terminate_due_to_error(
                format!("transaction {} errored: {}", self.id, error),
                None,
            ),
        }
    }

    //TODO: use proper error type here
    pub async fn transport_error(&mut self, reason: String) {
        self.terminate_due_to_error(reason, None);
    }

    async fn next_step(&mut self) -> Result<(), Error> {
        match &self.state {
            TrxState::Completed(completed) => {
                match (completed.has_timedout(), completed.should_retransmit()) {
                    (true, _) => self.terminate_due_to_timeout(),
                    (false, true) => {
                        self.handlers
                            .transport
                            .send(self.response.clone().into())
                            .await?;
                        self.state = TrxState::Completed(completed.retransmit());
                    }
                    (false, false) => (),
                }
            }
            TrxState::Accepted(accepted) => {
                if accepted.should_terminate() {
                    self.terminate_due_to_timeout();
                }
            }
            TrxState::Confirmed(confirmed) => {
                if confirmed.should_terminate() {
                    self.terminate();
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
                self.handlers
                    .transport
                    .send(self.response.clone().into())
                    .await?;
            }
            (TrxState::Completed(_), Method::Invite) => {
                self.handlers
                    .transport
                    .send(self.response.clone().into())
                    .await?;
            }
            (TrxState::Completed(_), Method::Ack) => {
                self.confirm(request);
            }
            (TrxState::Accepted(_), Method::Invite) => {
                self.handlers
                    .transport
                    .send(self.response.clone().into())
                    .await?;
            }
            (TrxState::Accepted(_), Method::Ack) => {
                self.handlers.tu.process(request.into()).await?;
            }
            (TrxState::Confirmed(_), Method::Ack) => {
                //absorb ack
            }
            _ => self.terminate_due_to_error(
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
                self.handlers.transport.send(response.into()).await?;
            }
            (TrxState::Proceeding(_), StatusCodeKind::Successful) => {
                self.response = response.clone();
                self.handlers.transport.send(response.into()).await?;
                self.accept();
            }
            (TrxState::Proceeding(_), _) => {
                self.response = response.clone();
                self.handlers.transport.send(response.into()).await?;
                self.complete();
            }
            (TrxState::Accepted(_), StatusCodeKind::Successful) => {
                self.handlers
                    .transport
                    .send(self.response.clone().into())
                    .await?;
            }
            _ => self.terminate_due_to_error(
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

    //TODO: check if we should get a response here
    fn terminate(&mut self) {
        self.state = TrxState::Terminated(Terminated::Expected {
            //response: ...
            entered_at: Instant::now(),
        });
    }

    //TODO: check if we can get a response here
    fn terminate_due_to_timeout(&mut self) {
        self.state = TrxState::Terminated(Terminated::TimedOut {
            response: None,
            entered_at: Instant::now(),
        });
    }

    fn terminate_due_to_error(&mut self, error: String, sip_message: Option<rsip::SipMessage>) {
        self.state = TrxState::Terminated(Terminated::Errored {
            entered_at: Instant::now(),
            sip_message,
            error,
        });
    }
}
