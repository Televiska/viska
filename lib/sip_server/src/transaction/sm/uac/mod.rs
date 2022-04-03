mod states;

pub use states::{Completed, Errored, Proceeding, Terminated, Trying};

use crate::Error;
use common::{
    rsip::{self, message::HeadersExt},
    tokio::time::Instant,
};
use models::{rsip_ext::*, transaction::TransactionId, Handlers};

//TODO: add state checks as well for better guarantees, look at dialogs

//should come from config
pub static TIMER_T1: u64 = 500;
pub static TIMER_T2: u64 = 4000;
pub static TIMER_F: u64 = 64 * TIMER_T1;

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
    Errored(Errored),
}

impl std::fmt::Display for TrxState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Trying(_) => write!(f, "TrxState::Trying"),
            Self::Proceeding(_) => write!(f, "TrxState::Proceeding"),
            Self::Completed(_) => write!(f, "TrxState::Completed"),
            Self::Terminated(_) => write!(f, "TrxState::Terminated"),
            Self::Errored(_) => write!(f, "TrxState::Errored"),
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
            Err(error) => self.error(format!("transaction {} errored: {}", self.id, error), None),
        };
    }

    pub fn is_active(&self) -> bool {
        !matches!(self.state, TrxState::Errored(_) | TrxState::Terminated(_))
    }

    //TODO: use proper error type here
    pub async fn transport_error(&mut self, reason: String) {
        self.error(reason, None);
    }

    async fn next_step(&mut self) -> Result<(), Error> {
        match &self.state {
            TrxState::Trying(trying) => match (trying.has_timedout(), trying.should_retransmit()) {
                (true, _) => self.terminate(),
                (false, true) => {
                    self.handlers
                        .transport
                        .send(self.request.clone().into())
                        .await?;
                    self.state = TrxState::Trying(trying.retransmit());
                }
                (false, false) => (),
            },
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
                self.proceed(response);
            }
            (TrxState::Trying(_), _) => {
                self.handlers.tu.process(response.clone().into()).await?;
                self.complete(response);
            }
            (TrxState::Proceeding(_), StatusCodeKind::Provisional) => {
                self.handlers.tu.process(response.clone().into()).await?;
                self.update_response(response);
            }
            (TrxState::Proceeding(_), _) => {
                self.handlers.tu.process(response.clone().into()).await?;
                self.complete(response);
            }
            (TrxState::Completed(_), kind) if kind > StatusCodeKind::Successful => {
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

    fn terminate(&mut self) {
        unimplemented!("");
    }

    fn error(&mut self, error: String, response: Option<rsip::Response>) {
        self.state = TrxState::Errored(Errored {
            entered_at: Instant::now(),
            response,
            error,
        });
    }
}
