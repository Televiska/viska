mod states;

pub use states::{Completed, Proceeding, Terminated, Trying};

use crate::Error;
use common::{
    rsip::{self, message::HeadersExt},
    tokio::time::Instant,
};
use models::{transaction::TransactionId, Handlers};

//TODO: add state checks as well for better guarantees, look at dialogs

//should come from config
pub static TIMER_T1: u64 = 500;
pub static TIMER_T2: u64 = 4000;
pub static TIMER_F: u64 = 64 * TIMER_T1;
pub static TIMER_K: u64 = 5000;

//implements RFC6026 as well
#[allow(dead_code)]
#[derive(Debug)]
pub struct TrxStateMachine {
    pub id: TransactionId,
    pub state: TrxState,
    pub request: rsip::Request,
    pub created_at: Instant,
    handlers: Handlers,
}

#[derive(Debug)]
pub enum TrxState {
    Trying(Trying),
    Proceeding(Proceeding),
    Completed(Completed),
    Terminated(Terminated),
}

impl std::fmt::Display for TrxState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Trying(_) => write!(f, "TrxState::Trying"),
            Self::Proceeding(_) => write!(f, "TrxState::Proceeding"),
            Self::Completed(_) => write!(f, "TrxState::Completed"),
            Self::Terminated(_) => write!(f, "TrxState::Terminated"),
        }
    }
}

impl TrxStateMachine {
    pub fn new(handlers: Handlers, request: rsip::Request) -> Result<Self, Error> {
        Ok(Self {
            id: request.transaction_id()?.expect("transaction_id"),
            state: TrxState::Trying(Default::default()),
            request,
            created_at: Instant::now(),
            handlers,
        })
    }

    pub async fn next(&mut self, response: Option<rsip::Response>) {
        let result = match response {
            Some(response) => self.next_step_with(response).await,
            None => self.next_step().await,
        };

        match result {
            Ok(()) => (),
            Err(error) => self.terminate_due_to_error(
                format!("transaction {} errored: {}", self.id, error),
                None,
            ),
        };
    }

    pub fn is_active(&self) -> bool {
        !matches!(self.state, TrxState::Terminated(_))
    }

    //TODO: use proper error type here
    pub async fn transport_error(&mut self, reason: String) {
        self.terminate_due_to_error(reason, None);
    }

    async fn next_step(&mut self) -> Result<(), Error> {
        match &self.state {
            TrxState::Trying(trying) => match (trying.has_timedout(), trying.should_retransmit()) {
                (true, _) => self.terminate_due_to_timeout(),
                (false, true) => {
                    self.handlers
                        .transport
                        .send(self.request.clone().into())
                        .await?;
                    self.state = TrxState::Trying(trying.retransmit());
                }
                (false, false) => (),
            },
            TrxState::Proceeding(proceeding) => {
                match (proceeding.has_timedout(), proceeding.should_retransmit()) {
                    (true, _) => self.terminate_due_to_timeout(),
                    (false, true) => {
                        self.handlers
                            .transport
                            .send(self.request.clone().into())
                            .await?;
                        self.state = TrxState::Proceeding(proceeding.retransmit());
                    }
                    (false, false) => (),
                }
            }
            TrxState::Completed(completed) => {
                if completed.should_terminate() {
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
            (TrxState::Trying(_), StatusCodeKind::Provisional) => {
                self.handlers.tu.process(response.clone().into()).await?;
                self.proceed();
            }
            (TrxState::Trying(_), _) => {
                self.handlers.tu.process(response.into()).await?;
                self.complete();
            }
            (TrxState::Proceeding(_), StatusCodeKind::Provisional) => {
                self.handlers.tu.process(response.into()).await?;
            }
            (TrxState::Proceeding(_), _) => {
                self.handlers.tu.process(response.into()).await?;
                self.complete();
            }
            (TrxState::Completed(_), kind) if kind > StatusCodeKind::Successful => {
                //quench retransmission
            }
            (_, _) => {
                self.terminate_due_to_error(
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

    fn proceed(&mut self) {
        let trying = match self.state {
            TrxState::Trying(trying) => trying,
            _ => {
                return self.terminate_due_to_error(
                    format!("can't move to Proceeding state from state: {}", self.state),
                    None,
                );
            }
        };
        self.state = TrxState::Proceeding(Proceeding::from(trying));
    }

    fn complete(&mut self) {
        self.state = TrxState::Completed(Completed {
            entered_at: Instant::now(),
        });
    }

    fn terminate(&mut self) {
        self.state = TrxState::Terminated(Terminated::Expected {
            entered_at: Instant::now(),
        });
    }

    fn terminate_due_to_timeout(&mut self) {
        self.state = TrxState::Terminated(Terminated::TimedOut {
            response: None,
            entered_at: Instant::now(),
        });
    }

    fn terminate_due_to_error(&mut self, error: String, response: Option<rsip::Response>) {
        self.state = TrxState::Terminated(Terminated::Errored {
            entered_at: Instant::now(),
            response,
            error,
        });
    }
}
