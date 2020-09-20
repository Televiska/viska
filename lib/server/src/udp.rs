use common::bytes::Bytes;
use common::futures::stream::{SplitSink, SplitStream};
use common::futures::SinkExt;
use common::futures_util::stream::StreamExt;
use common::tokio_util::codec::BytesCodec;
use common::tokio_util::udp::UdpFramed;
use models::{server::UdpTuple, ChannelOf};
//use processor::Processor;
use std::net::SocketAddr;
use tokio::net::UdpSocket;
use tokio::sync::mpsc::{self, Receiver, Sender};

type UdpSink = SplitSink<UdpFramed<BytesCodec>, (Bytes, SocketAddr)>;
type UdpStream = SplitStream<UdpFramed<BytesCodec>>;

pub struct UdpServer {
    #[allow(dead_code)]
    transport_sink: Sender<UdpTuple>,
    udp_sink: UdpSink,
    udp_stream: UdpStream,
    server_stream: Receiver<UdpTuple>,
}

// listens to server_stream and forwards to udp_sink
// listens to udp_stream and forwards to transport_sink
impl UdpServer {
    pub async fn new() -> Result<Self, crate::Error> {
        let (server_sink, server_stream): ChannelOf<UdpTuple> = mpsc::channel(100);

        let (udp_sink, udp_stream) = create_socket().await?;

        let transport_sink = processor::transport::Transport::spawn(server_sink.clone()).await?;

        Ok(Self {
            udp_sink,
            transport_sink,
            udp_stream,
            server_stream,
        })
    }

    pub async fn run(&mut self) {
        //this can be optimized further by having each arm on its own tokio spawn
        //specifcally the first arm can be spawned since it doesn't depend on self (transport_sink
        //can be cloned)
        loop {
            tokio::select! {
                Some(request) = self.udp_stream.next() => {
                    match request {
                        Ok((request, addr)) => {
                            if self.transport_sink.send((request.freeze(), addr).into()).await.is_err() {
                                common::log::error!("failed to send to transport layer");
                            }
                        }
                        Err(e) => common::log::error!("{:?}", e),
                    }
                }
                Some(udp_tuple) = self.server_stream.next() => {
                    if self.udp_sink.send(udp_tuple.into()).await.is_err() {
                        common::log::error!("failed to send to udp socket");
                    }
                }
            }
        }
    }
}

async fn create_socket() -> Result<(UdpSink, UdpStream), crate::Error> {
    let socket = UdpSocket::bind("0.0.0.0:5060").await?;
    common::log::debug!("starting udp server listening in port 5060");
    let socket = UdpFramed::new(socket, BytesCodec::new());
    Ok(socket.split())
}
