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

use processor::{core::CoreLayer, transaction::TransactionLayer, transport::TransportLayer};

type UdpSink = SplitSink<UdpFramed<BytesCodec>, (Bytes, SocketAddr)>;
type UdpStream = SplitStream<UdpFramed<BytesCodec>>;

#[allow(dead_code)]
pub struct UdpServer {
    udp_sink: UdpSink,
    udp_stream: UdpStream,
    self_to_transport_sink: Sender<UdpTuple>,
    transport_to_self_sink: Sender<UdpTuple>,
    transport_to_self_stream: Receiver<UdpTuple>,
}

// listens to server_stream and forwards to udp_sink
// listens to udp_stream and forwards to transport_sink
impl UdpServer {
    pub async fn new<TR: TransportLayer, C: CoreLayer, TC: TransactionLayer>(
    ) -> Result<Self, crate::Error> {
        let (transport_to_self_sink, transport_to_self_stream): ChannelOf<UdpTuple> =
            mpsc::channel(100);

        let (udp_sink, udp_stream) = create_socket().await?;

        let self_to_transport_sink = TR::spawn::<C, TC>(transport_to_self_sink.clone()).await?;

        Ok(Self {
            udp_sink,
            udp_stream,
            self_to_transport_sink,
            transport_to_self_sink,
            transport_to_self_stream,
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
                            println!("########################################################################");
                            println!("{}", String::from_utf8(request.to_vec()).expect("utf bytes to string"));
                            println!("########################################################################");
                            if self.self_to_transport_sink.send((request.freeze(), addr).into()).await.is_err() {
                                common::log::error!("failed to send to transport layer");
                            }
                        }
                        Err(e) => common::log::error!("{:?}", e),
                    }
                }
                Some(udp_tuple) = self.transport_to_self_stream.next() => {
                    println!("########################################################################");
                    println!("{}", String::from_utf8(udp_tuple.bytes.to_vec()).expect("utf bytes to string"));
                    println!("########################################################################");
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
