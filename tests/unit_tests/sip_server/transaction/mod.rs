pub mod uac_tests;
pub mod uas_tests;

use crate::common::snitches::SpySnitch;
use models::{transport::TransportLayerMsg, tu::TuLayerMsg};
use sip_server::Transaction;

pub async fn setup() -> (
    SpySnitch<TuLayerMsg>,
    Transaction,
    SpySnitch<TransportLayerMsg>,
) {
    let (handlers, receivers) = models::channels_builder();
    let transport = SpySnitch::new(handlers.clone(), receivers.transport).expect("transport");
    let transaction =
        Transaction::new(handlers.clone(), receivers.transaction).expect("transaction");
    let tu = SpySnitch::new(handlers.clone(), receivers.tu).expect("tu");

    (tu, transaction, transport)
}
