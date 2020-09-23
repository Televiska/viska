use common::async_trait::async_trait;
use common::futures_util::stream::StreamExt;
use models::{transaction::TransactionMsg, transport::TransportMsg, ChannelOf};
use tokio::sync::mpsc::{self, Receiver, Sender};

#[allow(dead_code)]
pub struct Transaction {
    core_to_self_sink: Sender<TransactionMsg>,
    self_to_core_sink: Sender<TransactionMsg>,
    transport_to_self_sink: Sender<TransportMsg>,
    self_to_transport_sink: Sender<TransportMsg>,
}

#[async_trait]
pub trait TransactionLayer: Send + Sync {
    async fn spawn(
        self_to_transport_sink: Sender<TransportMsg>,
        self_to_core_sink: Sender<TransactionMsg>,
    ) -> Result<(Sender<TransactionMsg>, Sender<TransportMsg>), crate::Error>;
}

// listens to transport_stream and might forwards to core_sink, or respond back to transport_sink
// listens to core_stream and forwards to transport_sink
#[async_trait]
impl TransactionLayer for Transaction {
    async fn spawn(
        self_to_transport_sink: Sender<TransportMsg>,
        self_to_core_sink: Sender<TransactionMsg>,
    ) -> Result<(Sender<TransactionMsg>, Sender<TransportMsg>), crate::Error> {
        let (core_to_self_sink, core_to_self_stream): ChannelOf<TransactionMsg> =
            mpsc::channel(100);

        let (transport_to_self_sink, transport_to_self_stream): ChannelOf<TransportMsg> =
            mpsc::channel(100);

        let core_to_self_sink_cloned = core_to_self_sink.clone();
        let transport_to_self_sink_cloned = transport_to_self_sink.clone();
        tokio::spawn(async move {
            let mut transaction = Self {
                core_to_self_sink,
                self_to_core_sink,
                transport_to_self_sink,
                self_to_transport_sink,
            };
            transaction
                .run(core_to_self_stream, transport_to_self_stream)
                .await;
        });

        Ok((core_to_self_sink_cloned, transport_to_self_sink_cloned))
    }
}

impl Transaction {
    async fn run(
        &mut self,
        mut core_to_self_stream: Receiver<TransactionMsg>,
        mut transport_to_self_stream: Receiver<TransportMsg>,
    ) {
        use std::convert::TryInto;

        let mut self_to_transport_sink = self.self_to_transport_sink.clone();
        let mut self_to_core_sink = self.self_to_core_sink.clone();

        loop {
            tokio::select! {
                Some(transaction_msg) = core_to_self_stream.next() => {
                    if self_to_transport_sink.send(transaction_msg.into()).await.is_err() {
                        common::log::error!("failed to send");
                    }
                }
                Some(transport_msg) = transport_to_self_stream.next() => {
                    let transaction_msg = transport_msg.try_into();
                    match transaction_msg {
                        Ok(transaction_msg) => {
                            if self_to_core_sink.send(transaction_msg).await.is_err() {
                                common::log::error!("failed to send");
                            }
                        },
                        Err(error) => {
                            common::log::error!("failed to convert to transaction msg: {:?}", error)
                        }
                    }
                }
            }
        }
    }
}
