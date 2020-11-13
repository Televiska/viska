pub mod processor;
pub mod uac;
pub mod uas;

use crate::{Error, SipManager};
use common::async_trait::async_trait;
use std::sync::{Arc, Weak};

use common::bytes::Bytes;
use common::futures::stream::{SplitSink, SplitStream};
use common::futures::SinkExt;
use common::futures_util::stream::StreamExt;
use common::tokio_util::codec::BytesCodec;
use common::tokio_util::udp::UdpFramed;
use models::{server::UdpTuple, transport::TransportMsg};
//use processor::Processor;
use std::any::Any;
use std::net::SocketAddr;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;

type UdpSink = SplitSink<UdpFramed<BytesCodec>, (Bytes, SocketAddr)>;
type UdpStream = SplitStream<UdpFramed<BytesCodec>>;

#[async_trait]
pub trait TransportLayer: Send + Sync + Any {
    fn new(sip_manager: Weak<SipManager>) -> Result<Self, Error>
    where
        Self: Sized;
    async fn process_incoming_message(&self, udp_tuple: UdpTuple) -> Result<(), Error>;
    async fn send(&self, msg: TransportMsg) -> Result<(), Error>;
    async fn run(&self);
    fn sip_manager(&self) -> Arc<SipManager>;
    fn as_any(&self) -> &dyn Any;
}

//with tokio 3.x, the Mutexes will be replaced with an Arc here
pub struct Transport {
    sip_manager: Weak<SipManager>,
    pub processor: processor::Processor,
    udp_sink: Mutex<UdpSink>,
    udp_stream: Mutex<UdpStream>,
}

#[async_trait]
impl TransportLayer for Transport {
    fn new(sip_manager: Weak<SipManager>) -> Result<Self, Error> {
        let (udp_sink, udp_stream) = create_socket()?;

        Ok(Self {
            sip_manager,
            processor: processor::Processor::default(),
            udp_sink: Mutex::new(udp_sink),
            udp_stream: Mutex::new(udp_stream),
        })
    }

    async fn process_incoming_message(&self, udp_tuple: UdpTuple) -> Result<(), Error> {
        use std::convert::TryInto;

        let message = self
            .processor
            .process_incoming_message(udp_tuple.try_into()?)
            .await?;

        self.sip_manager()
            .core
            .process_incoming_message(message)
            .await;

        Ok(())
    }

    async fn send(&self, msg: TransportMsg) -> Result<(), Error> {
        common::log::debug!("{:?}", msg);

        Ok(self
            .udp_send(self.processor.process_outgoing_message(msg).into())
            .await?)
    }

    async fn run(&self) {
        loop {
            match self.udp_stream.lock().await.next().await {
                Some(Ok((request, addr))) => {
                    //debug_message(request.to_vec());

                    match self
                        .process_incoming_message((request.freeze(), addr).into())
                        .await
                    {
                        Ok(_) => (),
                        Err(error) => {
                            common::log::error!("failed to process incoming message: {:?}", error)
                        }
                    }
                }
                Some(Err(e)) => common::log::error!("{:?}", e),
                None => common::log::error!("nothing here"),
            }
        }
    }

    fn sip_manager(&self) -> Arc<SipManager> {
        self.sip_manager.upgrade().expect("sip manager is missing!")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Transport {
    async fn udp_send(&self, udp_tuple: UdpTuple) -> Result<(), Error> {
        //debug_message(udp_tuple.bytes.to_vec());

        Ok(self.udp_sink.lock().await.send(udp_tuple.into()).await?)
    }
}

fn create_socket() -> Result<(UdpSink, UdpStream), crate::Error> {
    let socket = UdpSocket::from_std(std::net::UdpSocket::bind("0.0.0.0:5060")?)?;
    common::log::debug!("starting udp server listening in port 5060");
    let socket = UdpFramed::new(socket, BytesCodec::new());
    Ok(socket.split())
}

#[allow(dead_code)]
fn debug_message(bytes: Vec<u8>) {
    let separator = "########################################################################";
    let message = String::from_utf8(bytes).expect("utf bytes to string");
    println!("{}\n{}\n{}", separator, message, separator);
}
