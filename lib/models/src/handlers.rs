use crate::{
    transaction::{TransactionHandler, TransactionLayerMsg},
    transport::{TransportHandler, TransportLayerMsg},
    tu::{TuHandler, TuLayerMsg},
};
use common::tokio::sync::mpsc::Sender;

#[derive(Debug, Clone)]
pub struct Handlers {
    pub tu: TuHandler,
    pub transaction: TransactionHandler,
    pub transport: TransportHandler,
}

impl
    From<(
        Sender<TuLayerMsg>,
        Sender<TransactionLayerMsg>,
        Sender<TransportLayerMsg>,
    )> for Handlers
{
    fn from(
        from: (
            Sender<TuLayerMsg>,
            Sender<TransactionLayerMsg>,
            Sender<TransportLayerMsg>,
        ),
    ) -> Self {
        Self {
            tu: from.0.into(),
            transaction: from.1.into(),
            transport: from.2.into(),
        }
    }
}
/*
pub struct Receivers {
    pub tu: Receiver<TuLayerMsg>,
    pub transaction: Receiver<TransactionLayerMsg>,
    pub transport: Receiver<TransportLayerMsg>,
}

impl
    From<(
        crate::TuReceiver,
        crate::TrxReceiver,
        crate::TrReceiver,
    )> for Receivers
{
    fn from(
        from: (
            crate::TuReceiver,
            crate::TrxReceiver,
            crate::TrReceiver,
        ),
    ) -> Self {
        Self {
            tu: from.0,
            transaction: from.1,
            transport: from.2,
        }
    }
}*/
