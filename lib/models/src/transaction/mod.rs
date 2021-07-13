use crate::transport::TransportMsg;
use common::tokio::time::Instant;

#[allow(dead_code)]
#[derive(Debug)]
pub struct ClientTransactionData {
    pub id: String,
    pub state: ClientTransactionState,
    pub msg: TransportMsg,
    pub created_at: Instant,
}

#[derive(Debug)]
pub enum ClientTransactionState {
    Calling,
    Proceeding,
    Completed,
    Accepted,
    Terminated,
    Errored,
}
