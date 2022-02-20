use crate::{transaction::TransactionHandler, transport::TransportHandler, tu::TuHandler};

#[derive(Debug, Clone)]
pub struct Handlers {
    pub tu: TuHandler,
    pub transaction: TransactionHandler,
    pub transport: TransportHandler,
}
