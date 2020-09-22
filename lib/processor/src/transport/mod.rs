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
}

// listens to core_to_self_stream and forwards to self_to_server_sink
// listens to server_to_self_stream and forwards to self_to_core_sink
impl Transport {
    pub async fn spawn(
        self_to_server_sink: Sender<UdpTuple>,
    ) -> Result<Sender<UdpTuple>, crate::Error> {
        let (core_to_self_sink, core_to_self_stream): ChannelOf<TransportMsg> = mpsc::channel(100);

        let (transaction_to_self_sink, transaction_to_self_stream): ChannelOf<TransportMsg> =
            mpsc::channel(100);

        let (server_to_self_sink, server_to_self_stream): ChannelOf<UdpTuple> = mpsc::channel(100);

        let (self_to_core_sink, self_to_transaction_sink) =
            crate::core::Core::spawn(core_to_self_sink.clone(), transaction_to_self_sink.clone())
                .await?;

        let server_to_self_sink_cloned = server_to_self_sink.clone();
        tokio::spawn(async move {
            let mut transport = Self {
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

    async fn run(
        &mut self,
        mut server_to_self_stream: Receiver<UdpTuple>,
        mut transaction_to_self_stream: Receiver<TransportMsg>,
        mut core_to_self_stream: Receiver<TransportMsg>,
    ) {
        use std::convert::TryInto;

        let mut self_to_server_sink = self.self_to_server_sink.clone();
        let mut self_to_core_sink = self.self_to_core_sink.clone();

        loop {
            tokio::select! {
                Some(udp_tuple) = server_to_self_stream.next() => {
                    let transport_msg = udp_tuple.try_into();
                    match transport_msg {
                        Ok(transport_msg) => {
                            if self_to_core_sink.send(transport_msg).await.is_err() {
                                common::log::error!("failed to send");
                            }
                        },
                        Err(error) => {
                            common::log::error!("failed to convert to transport msg: {:?}", error)
                        }
                    }
                }
                Some(transport_tuple) = transaction_to_self_stream.next() => {
                    if self_to_server_sink.send(transport_tuple.into()).await.is_err() {
                        common::log::error!("failed to send");
                    }
                }
                Some(transport_tuple) = core_to_self_stream.next() => {
                    if self_to_server_sink.send(transport_tuple.into()).await.is_err() {
                        common::log::error!("failed to send");
                    }
                }
            }
        }
    }
}
