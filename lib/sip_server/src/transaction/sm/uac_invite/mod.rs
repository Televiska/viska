mod states;

pub use states::{Accepted, Calling, Completed, Proceeding, Terminated};

use crate::Error;
use common::{
    rsip::{self, message::HeadersExt},
    tokio::time::Instant,
};
use models::{rsip_ext::*, transaction::TransactionId, Handlers};

//TODO: add state checks as well for better guarantees, look at dialogs

//should come from config
pub static TIMER_T1: u64 = 500;
pub static TIMER_B: u64 = 64 * TIMER_T1;
pub static TIMER_M: u64 = 64 * TIMER_T1;
pub static TIMER_D: u64 = 32000;

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
    Calling(Calling),
    Proceeding(Proceeding),
    Completed(Completed),
    Accepted(Accepted),
    Terminated(Terminated),
}

impl std::fmt::Display for TrxState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Calling(_) => write!(f, "TrxState::Calling"),
            Self::Proceeding(_) => write!(f, "TrxState::Proceeding"),
            Self::Completed(_) => write!(f, "TrxState::Completed"),
            Self::Accepted(_) => write!(f, "TrxState::Accepted"),
            Self::Terminated(_) => write!(f, "TrxState::Terminated"),
        }
    }
}

impl TrxStateMachine {
    pub fn new(handlers: Handlers, request: rsip::Request) -> Result<Self, Error> {
        Ok(Self {
            id: request.transaction_id()?.expect("transaction_id"),
            state: TrxState::Calling(Default::default()),
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

    //TODO: can you investigate what will happen if the SM is in proceeding state
    //but nothing comes from the peer? Will it stuck in proceeding forever ?
    async fn next_step(&mut self) -> Result<(), Error> {
        match &self.state {
            TrxState::Calling(calling) => {
                match (calling.has_timedout(), calling.should_retransmit()) {
                    (true, _) => self.terminate_due_to_timeout(),
                    (false, true) => {
                        self.handlers
                            .transport
                            .send(self.request.clone().into())
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
                self.handlers.tu.process(response.into()).await?;
                self.proceed();
            }
            (TrxState::Calling(_), StatusCodeKind::Successful) => {
                self.handlers.tu.process(response.into()).await?;
                self.accept();
            }
            (TrxState::Calling(_), _) => {
                self.handlers.tu.process(response.into()).await?;
                self.complete();
            }
            (TrxState::Proceeding(_), StatusCodeKind::Provisional) => {
                self.handlers.tu.process(response.into()).await?;
            }
            (TrxState::Proceeding(_), StatusCodeKind::Successful) => {
                self.handlers.tu.process(response.into()).await?;
                self.accept();
            }
            (TrxState::Proceeding(_), _) => {
                self.handlers.tu.process(response.clone().into()).await?;
                self.send_ack_request_from(response).await?;
                self.complete();
            }
            (TrxState::Accepted(_), StatusCodeKind::Successful) => {
                self.handlers.tu.process(response.into()).await?;
            }
            (TrxState::Completed(_), kind) if kind > StatusCodeKind::Successful => {
                self.complete();
                self.send_ack_request_from(response).await?;
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
        self.state = TrxState::Proceeding(Proceeding {
            entered_at: Instant::now(),
        });
    }

    fn complete(&mut self) {
        self.state = TrxState::Completed(Completed {
            entered_at: Instant::now(),
        });
    }

    fn accept(&mut self) {
        self.state = TrxState::Accepted(Accepted {
            entered_at: Instant::now(),
        });
    }

    fn terminate(&mut self) {
        self.state = TrxState::Terminated(Terminated::Expected {
            entered_at: Instant::now(),
        });
    }

    //TODO: if we end up here from proceeding, we should probably save the Response as well
    fn terminate_due_to_timeout(&mut self) {
        self.state = TrxState::Terminated(Terminated::TimedOut {
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

    async fn send_ack_request_from(&self, response: rsip::Response) -> Result<(), Error> {
        Ok(self
            .handlers
            .transport
            .send(self.request.ack_request_from(response).into())
            .await?)
    }
}
