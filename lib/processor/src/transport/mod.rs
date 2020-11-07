mod processor;
mod uac;
mod uas;

pub use self::processor::Processor;

use crate::core::CoreLayer;
use crate::transaction::TransactionLayer;
use common::async_trait::async_trait;
use common::futures_util::stream::StreamExt;
use models::{server::UdpTuple, transport::TransportMsg, ChannelOf};
use tokio::sync::mpsc::{self, Receiver, Sender};

#[allow(dead_code)]
pub struct Transport {
    core_to_self_sink: Sender<TransportMsg>,
    self_to_core_sink: Sender<TransportMsg>,
    transaction_to_self_sink: Sender<TransportMsg>,
    self_to_transaction_sink: Sender<TransportMsg>,
    server_to_self_sink: Sender<UdpTuple>,
    self_to_server_sink: Sender<UdpTuple>,
    processor: processor::Processor,
}

#[async_trait]
pub trait TransportLayer: Send + Sync {
    async fn spawn<C: CoreLayer, T: TransactionLayer>(
        self_to_server_sink: Sender<UdpTuple>,
    ) -> Result<Sender<UdpTuple>, crate::Error>;
}

// listens to core_to_self_stream and forwards to self_to_server_sink
// listens to server_to_self_stream and forwards to self_to_core_sink
#[async_trait]
impl TransportLayer for Transport {
    async fn spawn<C: CoreLayer, T: TransactionLayer>(
        self_to_server_sink: Sender<UdpTuple>,
    ) -> Result<Sender<UdpTuple>, crate::Error> {
        let (core_to_self_sink, core_to_self_stream): ChannelOf<TransportMsg> = mpsc::channel(100);

        let (transaction_to_self_sink, transaction_to_self_stream): ChannelOf<TransportMsg> =
            mpsc::channel(100);

        let (server_to_self_sink, server_to_self_stream): ChannelOf<UdpTuple> = mpsc::channel(100);

        let (self_to_core_sink, self_to_transaction_sink) =
            C::spawn::<T>(core_to_self_sink.clone(), transaction_to_self_sink.clone()).await?;

        let server_to_self_sink_cloned = server_to_self_sink.clone();
        tokio::spawn(async move {
            let mut transport = Self {
                processor: processor::Processor::new(
                    self_to_core_sink.clone(),
                    self_to_transaction_sink.clone(),
                    self_to_server_sink.clone(),
                ),
                core_to_self_sink,
                self_to_core_sink,
                transaction_to_self_sink,
                self_to_transaction_sink,
                server_to_self_sink,
                self_to_server_sink,
            };
            transport
                .run(
                    server_to_self_stream,
                    transaction_to_self_stream,
                    core_to_self_stream,
                )
                .await;
        });

        Ok(server_to_self_sink_cloned)
    }
}

impl Transport {
    async fn run(
        &mut self,
        mut server_to_self_stream: Receiver<UdpTuple>,
        mut transaction_to_self_stream: Receiver<TransportMsg>,
        mut core_to_self_stream: Receiver<TransportMsg>,
    ) {
        use std::convert::TryInto;

        loop {
            tokio::select! {
                Some(udp_tuple) = server_to_self_stream.next() => {
                    let transport_msg = udp_tuple.try_into();
                    match transport_msg {
                        Ok(transport_msg) => {
                            self.processor.handle_server_message(transport_msg).await;
                        },
                        Err(error) => {
                            common::log::error!("failed to convert to transport msg: {:?}", error)
                        }
                    }
                }
                Some(transport_msg) = transaction_to_self_stream.next() => {
                    self.processor.handle_transaction_message(transport_msg).await;
                }
                Some(transport_msg) = core_to_self_stream.next() => {
                    self.processor.handle_core_message(transport_msg).await;
                }
            }
        }
    }
}
