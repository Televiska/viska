use super::TransportProcessor;
use crate::Error;
use common::async_trait::async_trait;
use models::transport::{RequestMsg, ResponseMsg};

//TODO: Processor should return an Option<T>
//so that if None, transport skips the message

#[derive(Debug, Default)]
pub struct Processor;

#[async_trait]
impl TransportProcessor for Processor {
    async fn process_outgoing_request(
        &self,
        RequestMsg {
            sip_request,
            peer,
            transport,
        }: RequestMsg,
    ) -> Result<Option<RequestMsg>, Error> {
        let sip_request = super::uac::apply_request_defaults(sip_request, peer, transport)?;

        Ok(Some(RequestMsg {
            sip_request,
            peer,
            transport,
        }))
    }

    async fn process_incoming_response(
        &self,
        ResponseMsg {
            sip_response,
            peer,
            transport,
        }: ResponseMsg,
    ) -> Result<Option<ResponseMsg>, Error> {
        let sip_response = super::uac::apply_response_defaults(sip_response, peer, transport)?;

        Ok(Some(ResponseMsg {
            sip_response,
            peer,
            transport,
        }))
    }

    async fn process_incoming_request(
        &self,
        RequestMsg {
            sip_request,
            peer,
            transport,
        }: RequestMsg,
    ) -> Result<Option<RequestMsg>, Error> {
        let sip_request = super::uas::apply_request_defaults(sip_request, peer, transport)?;

        Ok(Some(RequestMsg {
            sip_request,
            peer,
            transport,
        }))
    }

    async fn process_outgoing_response(
        &self,
        ResponseMsg {
            sip_response,
            peer,
            transport,
        }: ResponseMsg,
    ) -> Result<Option<ResponseMsg>, Error> {
        let sip_response = super::uas::apply_response_defaults(sip_response, peer, transport);

        Ok(Some(ResponseMsg {
            sip_response,
            peer,
            transport,
        }))
    }
}
