pub mod processor;
#[allow(clippy::module_inception)]
pub mod transport;
pub mod uac;
pub mod uas;

pub use processor::Processor;
pub use transport::Transport;

use crate::Error;
use common::async_trait::async_trait;
use models::transport::{RequestMsg, ResponseMsg};

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
