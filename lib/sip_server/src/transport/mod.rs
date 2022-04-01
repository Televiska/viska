//pub mod dns_lookup;
pub mod processor;
#[allow(clippy::module_inception)]
pub mod transport;
pub mod uac;
pub mod uas;

pub use processor::DefaultProcessor;
pub use transport::Transport;

use crate::Error;
use common::{async_trait::async_trait, rsip};
use models::transport::{RequestMsg, ResponseMsg, TransportMsg};

#[async_trait]
pub trait DnsLookup: Send + Sync + 'static {
    async fn request_msg_from(&self, request: rsip::Request) -> Result<RequestMsg, Error>;
    async fn response_msg_from(&self, response: rsip::Response) -> Result<ResponseMsg, Error>;
    async fn transport_msg_from(&self, message: rsip::SipMessage) -> Result<TransportMsg, Error> {
        match message {
            rsip::SipMessage::Request(request) => {
                self.request_msg_from(request).await.map(Into::into)
            }
            rsip::SipMessage::Response(response) => {
                self.response_msg_from(response).await.map(Into::into)
            }
        }
    }
}

#[async_trait]
pub trait TransportProcessor: Send + Sync + 'static {
    async fn process_outgoing_request(
        &self,
        RequestMsg {
            sip_request,
            peer,
            transport,
        }: RequestMsg,
    ) -> Result<Option<RequestMsg>, Error>;

    async fn process_incoming_response(
        &self,
        ResponseMsg {
            sip_response,
            peer,
            transport,
        }: ResponseMsg,
    ) -> Result<Option<ResponseMsg>, Error>;

    async fn process_incoming_request(
        &self,
        RequestMsg {
            sip_request,
            peer,
            transport,
        }: RequestMsg,
    ) -> Result<Option<RequestMsg>, Error>;

    async fn process_outgoing_response(
        &self,
        ResponseMsg {
            sip_response,
            peer,
            transport,
        }: ResponseMsg,
    ) -> Result<Option<ResponseMsg>, Error>;
}
