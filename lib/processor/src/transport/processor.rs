use crate::Error;
use models::{server::UdpTuple, transport::TransportMsg};
use rsip::SipMessage;
use tokio::sync::mpsc::Sender;

//transport processor
#[allow(dead_code)]
pub struct Processor {
    self_to_core_sink: Sender<TransportMsg>,
    self_to_transaction_sink: Sender<TransportMsg>,
    self_to_server_sink: Sender<UdpTuple>,
}

impl Processor {
    pub fn new(
        self_to_core_sink: Sender<TransportMsg>,
        self_to_transaction_sink: Sender<TransportMsg>,
        self_to_server_sink: Sender<UdpTuple>,
    ) -> Self {
        Self {
            self_to_core_sink,
            self_to_transaction_sink,
            self_to_server_sink,
        }
    }

    pub async fn handle_server_message(&self, msg: TransportMsg) {
        //let mut self_to_transaction_sink = self.self_to_transaction_sink.clone();
        let mut self_to_core_sink = self.self_to_core_sink.clone();

        let message = self.process_incoming_message(msg).await;

        match message {
            Ok(message) => {
                if self_to_core_sink.send(message).await.is_err() {
                    common::log::error!("failed to send");
                }
            }
            Err(error) => common::log::error!("failed process incoming message: {:?}", error),
        }
    }

    pub async fn handle_transaction_message(&self, msg: TransportMsg) {
        let mut self_to_server_sink = self.self_to_server_sink.clone();

        let message = self.process_outgoing_message(msg).await;

        if self_to_server_sink.send(message.into()).await.is_err() {
            common::log::error!("failed to send to server from transport processor");
        }
    }

    //TODO: consider merging that with transaction into something like `handle_outgoing_message`
    pub async fn handle_core_message(&self, msg: TransportMsg) {
        let mut self_to_server_sink = self.self_to_server_sink.clone();

        let message = self.process_outgoing_message(msg).await;

        if self_to_server_sink.send(message.into()).await.is_err() {
            common::log::error!("failed to send to server from transport processor");
        }
    }

    async fn process_incoming_message(&self, msg: TransportMsg) -> Result<TransportMsg, Error> {
        use super::{uac::apply_response_defaults, uas::apply_request_defaults};

        let TransportMsg {
            sip_message,
            peer,
            transport,
        } = msg;

        let sip_message = match sip_message {
            SipMessage::Request(request) => {
                apply_request_defaults(request, peer, transport.clone())?
            }
            SipMessage::Response(response) => {
                apply_response_defaults(response, peer, transport.clone())?
            }
        };

        Ok(TransportMsg {
            sip_message,
            peer,
            transport,
        })
    }

    async fn process_outgoing_message(&self, msg: TransportMsg) -> TransportMsg {
        use super::{uac::apply_request_defaults, uas::apply_response_defaults};

        let TransportMsg {
            sip_message,
            peer,
            transport,
        } = msg;

        let sip_message = match sip_message {
            SipMessage::Request(request) => {
                apply_request_defaults(request, peer, transport.clone())
            }
            SipMessage::Response(response) => {
                apply_response_defaults(response, peer, transport.clone())
            }
        };

        TransportMsg {
            sip_message,
            peer,
            transport,
        }
    }
}
