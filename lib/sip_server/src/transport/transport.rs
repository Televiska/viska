use super::{DnsLookup, TransportProcessor};

use crate::Error;
use std::{convert::TryInto, fmt::Debug, net::SocketAddr, sync::Arc};

use common::{
    bytes::Bytes,
    futures::{
        stream::{SplitSink, SplitStream},
        SinkExt,
    },
    futures_util::stream::StreamExt,
    rsip,
    tokio::{self, net::UdpSocket, sync::Mutex},
    tokio_util::codec::BytesCodec,
    tokio_util::udp::UdpFramed,
};
use models::{
    receivers::TrReceiver,
    transport::TransportLayerMsg,
    transport::{RequestMsg, ResponseMsg, TransportMsg, UdpTuple},
    Handlers,
};

type UdpSink = SplitSink<UdpFramed<BytesCodec>, (Bytes, SocketAddr)>;
type UdpStream = SplitStream<UdpFramed<BytesCodec>>;

#[derive(Debug)]
pub struct Transport<P: TransportProcessor, D: DnsLookup> {
    inner: Arc<Inner<P, D>>,
}

#[derive(Debug)]
pub struct Inner<P: TransportProcessor, D: DnsLookup> {
    processor: P,
    dns_lookup: D,
    udp_sink: Mutex<UdpSink>,
    handlers: Handlers,
}

impl<P: TransportProcessor, D: DnsLookup> Transport<P, D> {
    pub fn new(
        handlers: Handlers,
        processor: P,
        dns_lookup: D,
        messages_rx: TrReceiver,
    ) -> Result<Self, Error> {
        let (udp_sink, udp_stream) = create_socket()?;

        let me = Self {
            inner: Arc::new(Inner {
                processor,
                dns_lookup,
                udp_sink: Mutex::new(udp_sink),
                handlers,
            }),
        };

        me.run(messages_rx, udp_stream);

        Ok(me)
    }

    fn run(&self, messages: TrReceiver, udp_stream: UdpStream) {
        let inner = self.inner.clone();
        tokio::spawn(async move { inner.run(messages).await });
        let socket_inner = self.inner.clone();
        tokio::spawn(async move { socket_inner.run_socket(udp_stream).await });
    }
}

impl<P: TransportProcessor, D: DnsLookup> Inner<P, D> {
    async fn run(&self, mut messages: TrReceiver) {
        while let Some(request) = messages.recv().await {
            if let Err(err) = self.receive(request).await {
                common::log::error!("Error handling transport layer message: {}", err)
            }
        }
    }

    async fn udp_send(&self, udp_tuple: UdpTuple) -> Result<(), Error> {
        debug_message(udp_tuple.bytes.to_vec());

        Ok(self.udp_sink.lock().await.send(udp_tuple.into()).await?)
    }

    //TODO: here we don't spawn, could lead to deadlocks
    async fn receive(&self, msg: TransportLayerMsg) -> Result<(), Error> {
        match msg {
            TransportLayerMsg::Outgoing(msg) => self.receive_outgoing_message(msg).await?,
            TransportLayerMsg::Incoming(msg) => self.receive_incoming_message(msg).await?,
        };

        Ok(())
    }

    async fn receive_outgoing_message(&self, msg: rsip::SipMessage) -> Result<(), Error> {
        let TransportMsg {
            sip_message,
            peer,
            transport,
        } = self.dns_lookup.transport_msg_from(msg).await?;

        let msg: Option<TransportMsg> = match sip_message {
            rsip::SipMessage::Request(request) => self
                .processor
                .process_outgoing_request((request, peer, transport).into())
                .await?
                .map(Into::into),
            rsip::SipMessage::Response(response) => self
                .processor
                .process_outgoing_response((response, peer, transport).into())
                .await?
                .map(Into::into),
        };

        if let Some(transport_msg) = msg {
            //TODO: optimize clone here
            if let Err(err) = self.udp_send(transport_msg.clone().into()).await {
                self.report_transport_error(transport_msg, format!("{:?}", err))
                    .await?;
            }
        }

        Ok(())
    }

    async fn report_transport_error(&self, msg: TransportMsg, error: String) -> Result<(), Error> {
        let transaction_id = msg.transaction_id()?;

        match transaction_id {
            Some(transaction_id) => {
                if self
                    .handlers
                    .transaction
                    .has_transaction_for(transaction_id)
                    .await?
                {
                    self.handlers
                        .transaction
                        .transport_error(msg.sip_message, error)
                        .await?;
                } else {
                    self.handlers
                        .tu
                        .transport_error(msg.sip_message, error)
                        .await?;
                }
            }
            None => {
                self.handlers
                    .tu
                    .transport_error(msg.sip_message, error)
                    .await?;
            }
        };

        Ok(())
    }

    async fn receive_incoming_message(&self, udp_tuple: UdpTuple) -> Result<(), Error> {
        let TransportMsg {
            sip_message,
            peer,
            transport,
        } = udp_tuple.try_into()?;

        match sip_message {
            rsip::SipMessage::Request(request) => {
                if let Some(msg) = self
                    .processor
                    .process_incoming_request((request, peer, transport).into())
                    .await?
                {
                    self.process_incoming_request(msg).await?;
                }
            }
            rsip::SipMessage::Response(response) => {
                if let Some(msg) = self
                    .processor
                    .process_incoming_response((response, peer, transport).into())
                    .await?
                {
                    self.process_incoming_response(msg).await?;
                }
            }
        };

        Ok(())
    }

    async fn process_incoming_request(&self, request: RequestMsg) -> Result<(), Error> {
        Ok(self.handlers.tu.process(request.sip_request.into()).await?)
    }

    async fn process_incoming_response(&self, response: ResponseMsg) -> Result<(), Error> {
        let transaction_id = response.transaction_id()?;

        match transaction_id {
            Some(transaction_id) => {
                if self
                    .handlers
                    .transaction
                    .has_transaction_for(transaction_id)
                    .await?
                {
                    self.handlers
                        .transaction
                        .process(response.sip_response.into())
                        .await?;
                } else {
                    self.handlers
                        .tu
                        .process(response.sip_response.into())
                        .await?;
                }
            }
            None => {
                self.handlers
                    .tu
                    .process(response.sip_response.into())
                    .await?;
            }
        };

        Ok(())
    }

    async fn run_socket(&self, mut udp_stream: UdpStream) {
        loop {
            match udp_stream.next().await {
                Some(Ok((request, addr))) => {
                    debug_message(request.clone().freeze().to_vec());

                    match self
                        .receive(UdpTuple::from((request.freeze(), addr)).into())
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
    println!(
        "{}\n{}\n{}",
        separator,
        String::from_utf8_lossy(&bytes),
        separator
    );
    //let message = String::from_utf8(bytes).expect("utf bytes to string");
    //println!("{}\n{}\n{}", separator, message, separator);
}
