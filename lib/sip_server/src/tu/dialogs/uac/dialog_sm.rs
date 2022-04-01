use super::{
    states::{Confirmed, Early, Errored, Terminated, Unconfirmed},
    validations,
};

use crate::{presets, Error};
use common::rsip::{self, prelude::*, uri::UriWithParams};
use common::tokio::time::Instant;
use models::{rsip_ext::*, tu::DialogId, Handlers};

#[derive(Debug)]
pub struct DialogSm {
    pub id: DialogId,
    pub call_id: rsip::headers::CallId,
    pub transaction_id: String,
    pub local_tag: rsip::common::param::Tag,
    pub local_seqn: u32,
    pub local_uri: rsip::Uri,
    pub remote_tag: Option<rsip::common::param::Tag>,
    pub remote_seqn: Option<u32>,
    pub remote_uri: rsip::Uri,
    pub remote_target: Option<rsip::Uri>,
    pub secure: bool,
    pub route_set: Vec<UriWithParams>,
    pub session_type: SessionType,
    pub contact_header: rsip::headers::Contact,
    pub request: rsip::Request,
    pub state: DialogState,
    pub created_at: Instant,
    pub handlers: Handlers,
}

#[derive(Debug)]
pub enum SessionType {
    UacOffer,
    UasOffer,
    Other,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum DialogState {
    Unconfirmed(Unconfirmed), //just created
    Early(Early),             //received 1xx
    Confirmed(Confirmed),     //received 2xx
    Terminated(Terminated),
    Errored(Errored),
}

//TODO: remove unused async in private functions
//TODO: need to apply strict or loose routing (probably in a helper/module)
#[allow(dead_code)]
impl DialogSm {
    pub async fn new(handlers: Handlers, request: rsip::Request) -> Result<Self, Error> {
        validations::run(&request)?;

        let mut route_set: Vec<UriWithParams> = request
            .record_route_header()
            .map(|h| h.typed().map(|h| h.uris().to_owned()))
            .transpose()?
            .unwrap_or_default();
        route_set.reverse();

        //TODO: probably it is a good idea to save local_from and remote_to
        //and expose some attributes as fns on top of that
        let me = Self {
            id: request.dialog_id()?,
            call_id: request.call_id_header()?.clone(),
            transaction_id: request.transaction_id()?.expect("transaction id").into(),
            local_tag: request
                .from_header()?
                .tag()?
                .ok_or_else(|| Error::from("missing from tag"))?,
            local_seqn: request.cseq_header()?.seq()?,
            local_uri: request.from_header()?.uri()?,
            remote_tag: None,
            remote_seqn: None,
            remote_uri: request.to_header()?.uri()?,
            remote_target: None,
            route_set,
            session_type: session_type(&request)?,
            //TODO: need to check transport as well (arrived from TLS?)
            secure: request.uri.is_sips()?,
            contact_header: request.contact_header()?.clone(),
            request: request.clone(),
            state: DialogState::Unconfirmed(Default::default()),
            created_at: Instant::now(),
            handlers: handlers.clone(),
        };

        handlers.transaction.new_uac_invite(request).await?;

        Ok(me)
    }

    async fn _process_incoming_request(&mut self, request: rsip::Request) -> Result<(), Error> {
        if !matches!(self.state, DialogState::Confirmed(_)) {
            return Err(Error::custom(format!(
                "cannot process a request while UAC dialog state is in {}",
                self.state
            )));
        }

        self.validate_incoming_request(&request)?;
        match request.method {
            /*
            rsip::Method::Invite => {
                self.handlers
                    .transaction
                    .new_uas_invite(response_msg_from(request).await, None)
                    .await?
            }*/
            rsip::Method::Bye => {
                self.terminate(request.clone().into());
                self.handlers
                    .transaction
                    .new_uas(
                        request.clone(),
                        Some(presets::response_from(request, 200.into())?),
                    )
                    .await?
            }
            _ => self.error(
                format!(
                    "({}): don't know how to handle method {} inside a dialog",
                    self.id, request.method,
                ),
                Some(request.into()),
            ),
        }

        Ok(())
    }

    async fn _process_incoming_response(&mut self, response: rsip::Response) -> Result<(), Error> {
        match response.status_code().kind() {
            rsip::StatusCodeKind::Provisional => self.early(response).await,
            rsip::StatusCodeKind::Successful => {
                self.confirm(response.clone()).await?;
                self.handlers
                    .transport
                    .send(self.request.ack_request_from(response).into())
                    .await?;
            }
            rsip::StatusCodeKind::Redirection => self.error(
                format!(
                    "({}): received status {}, peer wants redirection to {}",
                    self.id,
                    response.status_code,
                    response
                        .contact_header()
                        .map(|h| h.to_string())
                        .unwrap_or_else(|_| "no contact header".into()),
                ),
                Some(response.into()),
            ),
            _ => self.error(
                format!(
                    "({}): unknown match: {}, {}",
                    self.id, response.status_code, self.state,
                ),
                Some(response.into()),
            ),
        };

        Ok(())
    }

    async fn _process_outgoing_request(&mut self, request: rsip::Request) -> Result<(), Error> {
        if !matches!(self.state, DialogState::Confirmed(_)) {
            return Err(Error::custom(format!(
                "cannot process a request while UAC dialog state is in {}",
                self.state
            )));
        }

        let request = self.set_outgoing_request_defaults_for(request)?;

        match request.method {
            rsip::Method::Invite => self.handlers.transaction.new_uac_invite(request).await?,
            rsip::Method::Bye => {
                self.terminate(request.clone().into());
                self.handlers.transaction.new_uac(request).await?
            }
            _ => self.error(
                format!(
                    "({}): don't know how to handle method {} inside a dialog",
                    self.id, request.method,
                ),
                Some(request.into()),
            ),
        }

        Ok(())
    }

    async fn _process_outgoing_response(&mut self, _: rsip::Response) -> Result<(), Error> {
        Ok(())
    }

    pub async fn transport_error(&mut self, reason: String, msg: rsip::SipMessage) {
        self.error(reason, Some(msg));
    }

    async fn early(&mut self, response: rsip::Response) {
        if !matches!(self.state, DialogState::Unconfirmed(_)) {
            return self.wrong_transition("early", response.into());
        }

        self.state = DialogState::Early(Early {
            response,
            entered_at: Instant::now(),
        });
    }

    async fn confirm(&mut self, response: rsip::Response) -> Result<(), Error> {
        if !matches!(
            self.state,
            DialogState::Early(_) | DialogState::Unconfirmed(_)
        ) {
            self.wrong_transition("confirm", response.into());
            return Ok(());
        }

        self.remote_tag = Some(
            response
                .to_header()?
                .typed()?
                .tag()
                .ok_or_else(|| Error::from("missing from tag"))?
                .clone(),
        );

        self.remote_seqn = Some(response.cseq_header()?.typed()?.seq);

        self.remote_target = Some(response.contact_header()?.typed()?.uri);

        self.state = DialogState::Confirmed(Confirmed {
            response,
            entered_at: Instant::now(),
        });

        Ok(())
    }

    //TODO: I suspect msg here should be an option
    fn terminate(&mut self, msg: rsip::SipMessage) {
        if matches!(self.state, DialogState::Errored(_)) {
            return self.wrong_transition("terminate", msg);
        }

        self.state = DialogState::Terminated(Terminated {
            entered_at: Instant::now(),
        });
    }

    fn error(&mut self, error: String, sip_message: Option<rsip::SipMessage>) {
        common::log::error!("Dialog {} errored: {}", self.id, error);
        self.state = DialogState::Errored(Errored {
            entered_at: Instant::now(),
            sip_message,
            error,
        });
    }

    fn wrong_transition(&mut self, desired_state: &'static str, msg: rsip::SipMessage) {
        self.error(
            format!(
                "({}), wrong transition: from {} to {}",
                self.id, self.state, desired_state
            ),
            Some(msg),
        );
    }

    fn increased_seqn(&mut self) -> u32 {
        self.local_seqn += 1;
        self.local_seqn
    }

    fn set_outgoing_request_defaults_for(
        &mut self,
        mut request: rsip::Request,
    ) -> Result<rsip::Request, Error> {
        request.from_header_mut()?.mut_tag(self.local_tag.clone())?;
        request.from_header_mut()?.mut_uri(self.local_uri.clone())?;

        request
            .to_header_mut()?
            .mut_tag(self.remote_tag.clone().expect("remote tag"))?;
        request.to_header_mut()?.mut_uri(self.remote_uri.clone())?;

        request.call_id_header_mut()?.replace(self.call_id.clone());
        if !matches!(request.method, rsip::Method::Ack | rsip::Method::Cancel) {
            request.cseq_header_mut()?.mut_seq(self.increased_seqn())?;
        }
        request.uri = self.remote_target.clone().expect("remote target");
        if !matches!(request.method, rsip::Method::Invite) {
            request
                .contact_header_mut()?
                .replace(self.contact_header.clone());
        }

        Ok(request)
    }

    fn validate_incoming_request(&mut self, request: &rsip::Request) -> Result<(), Error> {
        let req_seqn = request.cseq_header()?.seq()?;
        if let (Some(remote_seqn), req_seqn) = (self.remote_seqn, req_seqn) {
            if remote_seqn > req_seqn {
                return Err(Error::from(format!(
                    "request remote seqn is lower than {}",
                    remote_seqn
                )));
            }
        }
        //    (_, req_seqn) => self.remote_seqn = Some(req_seqn),
        //};
        self.remote_seqn = Some(req_seqn);

        Ok(())
    }

    pub async fn process_incoming_request(&mut self, request: rsip::Request) {
        if let Err(err) = self._process_incoming_request(request).await {
            self.error(
                format!(
                    "Dialog {} failed to process incoming request: {}",
                    self.id, err
                ),
                None,
            );
        }
    }

    pub async fn process_incoming_response(&mut self, response: rsip::Response) {
        if let Err(err) = self._process_incoming_response(response).await {
            self.error(
                format!(
                    "Dialog {} failed to process incoming response: {}",
                    self.id, err
                ),
                None,
            );
        }
    }

    pub async fn process_outgoing_request(&mut self, request: rsip::Request) {
        if let Err(err) = self._process_outgoing_request(request).await {
            self.error(
                format!(
                    "Dialog {} failed to process outgoing request: {}",
                    self.id, err
                ),
                None,
            );
        }
    }

    pub async fn process_outgoing_response(&mut self, response: rsip::Response) {
        if let Err(err) = self._process_outgoing_response(response).await {
            self.error(
                format!(
                    "Dialog {} failed to process outgoing response: {}",
                    self.id, err
                ),
                None,
            );
        }
    }
}

pub fn is_secure(request: &rsip::Request) -> Result<bool, Error> {
    Ok(request.uri.is_sips()?)
}

pub fn session_type(request: &rsip::Request) -> Result<SessionType, Error> {
    use rsip::header_opt;

    let is_session = header_opt!(request.headers().iter(), rsip::Header::ContentDisposition)
        .map(|header| header.typed().map(|h| h.is_session()))
        .transpose()?
        .unwrap_or(true);

    if is_session {
        //assuming the correct Cotent-Type here, maybe we should add a validation
        if request.body.is_empty() {
            Ok(SessionType::UasOffer)
        } else {
            Ok(SessionType::UacOffer)
        }
    } else {
        Ok(SessionType::Other)
    }
}

#[allow(dead_code)]
fn dialog_response_from(request: rsip::Request) -> Result<rsip::Response, crate::Error> {
    let mut headers: rsip::Headers = Default::default();
    headers.push(request.via_header()?.clone().into());
    headers.push(request.from_header()?.clone().into());
    headers.push(request.to_header()?.clone().into());
    headers.push(request.call_id_header()?.clone().into());
    headers.push(request.cseq_header()?.clone().into());
    headers.push(rsip::Header::ContentLength(Default::default()));
    headers.push(rsip::Header::Server(Default::default()));

    Ok(rsip::Response {
        headers,
        status_code: 404.into(),
        ..Default::default()
    })
}

impl std::fmt::Display for DialogState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unconfirmed(_) => write!(f, "DialogState::Unconfirmed"),
            Self::Early(_) => write!(f, "DialogState::Early"),
            Self::Confirmed(_) => write!(f, "DialogState::Confirmed"),
            Self::Terminated(_) => write!(f, "DialogState::Terminated"),
            Self::Errored(_) => write!(f, "DialogState::Errored"),
        }
    }
}
