use crate::Error;
use models::transport::{RequestMsg, ResponseMsg};

//transport processor

#[derive(Debug, Default)]
pub struct Processor;

impl Processor {
    pub async fn process_outgoing_request(
        &self,
        RequestMsg {
            sip_request,
            peer,
            transport,
        }: RequestMsg,
    ) -> Result<RequestMsg, Error> {
        let sip_request = super::uac::apply_request_defaults(sip_request, peer, transport)?;

        Ok(RequestMsg {
            sip_request,
            peer,
            transport,
        })
    }

    pub async fn process_incoming_response(
        &self,
        ResponseMsg {
            sip_response,
            peer,
            transport,
        }: ResponseMsg,
    ) -> Result<ResponseMsg, Error> {
        let sip_response = super::uac::apply_response_defaults(sip_response, peer, transport)?;

        Ok(ResponseMsg {
            sip_response,
            peer,
            transport,
        })
    }

    pub async fn process_incoming_request(
        &self,
        RequestMsg {
            sip_request,
            peer,
            transport,
        }: RequestMsg,
    ) -> Result<RequestMsg, Error> {
        let sip_request = super::uas::apply_request_defaults(sip_request, peer, transport)?;

        Ok(RequestMsg {
            sip_request,
            peer,
            transport,
        })
    }

    pub async fn process_outgoing_response(
        &self,
        ResponseMsg {
            sip_response,
            peer,
            transport,
        }: ResponseMsg,
    ) -> Result<ResponseMsg, Error> {
        let sip_response = super::uas::apply_response_defaults(sip_response, peer, transport);

        Ok(ResponseMsg {
            sip_response,
            peer,
            transport,
        })
    }
}
