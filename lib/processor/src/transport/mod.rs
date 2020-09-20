use common::futures_util::stream::StreamExt;
use models::{core::CoreMsg, server::UdpTuple, transport::TransportMsg, ChannelOf};
use tokio::sync::mpsc::{self, Receiver, Sender};

#[allow(dead_code)]
pub struct Transport {
    core_sink: Sender<CoreMsg>,
    server_sink: Sender<UdpTuple>,
    transport_sink: Sender<TransportMsg>,
}

// listens to transport_stream and forwards to server_sink
// listens to server_stream and forwards to core_sink
impl Transport {
    pub async fn spawn(server_sink: Sender<UdpTuple>) -> Result<Sender<UdpTuple>, crate::Error> {
        let (from_server_sink, server_stream): ChannelOf<UdpTuple> = mpsc::channel(100);

        let (transport_sink, transport_stream): ChannelOf<TransportMsg> = mpsc::channel(100);

        let core_sink = crate::core::Core::spawn(transport_sink.clone()).await?;

        tokio::spawn(async move {
            let mut transport = Self {
                core_sink,
                transport_sink,
                server_sink,
            };
            transport.run(server_stream, transport_stream).await;
        });

        Ok(from_server_sink)
    }

    async fn run(
        &mut self,
        mut server_stream: Receiver<UdpTuple>,
        mut transport_stream: Receiver<TransportMsg>,
    ) {
        use std::convert::TryInto;

        let mut server_sink = self.server_sink.clone();
        let mut core_sink = self.core_sink.clone();

        loop {
            tokio::select! {
                Some(udp_tuple) = server_stream.next() => {
                    let transport_msg = udp_tuple.try_into();
                    match transport_msg {
                        Ok(transport_msg) => {
                            if core_sink.send(transport_msg).await.is_err() {
                                common::log::error!("failed to send");
                            }
                        },
                        Err(error) => {
                            common::log::error!("failed to convert to transport msg: {:?}", error)
                        }
                    }
                }
                Some(transport_tuple) = transport_stream.next() => {
                    if server_sink.send(transport_tuple.into()).await.is_err() {
                        common::log::error!("failed to send");
                    }
                }
            }
        }
    }
}
