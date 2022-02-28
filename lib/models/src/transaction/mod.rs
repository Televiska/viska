mod transaction_handler;
mod transaction_layer_msg;

pub use transaction_handler::TransactionHandler;
pub use transaction_layer_msg::TransactionLayerMsg;

//TODO: reconsider
#[derive(Debug, Clone)]
pub struct TransactionData {
    pub id: i64,
    pub branch_id: String,
    pub dialog_id: i64,
}

//TODO: reconsider
#[derive(Debug, Clone)]
pub enum Registration {
    Trying(TransactionData),
    Proceeding(TransactionData),
    Completed(TransactionData),
    Terminated(TransactionData),
}

//TODO: reconsider
#[derive(Debug, Clone)]
pub enum NotFound {
    Default(TransactionData),
}
