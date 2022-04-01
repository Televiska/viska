use super::states::{Confirmed, Early, Errored, Terminated};

use crate::Error;
use common::rsip::{self, prelude::*, uri::UriWithParams};
use common::tokio::time::Instant;
use models::{rsip_ext::*, transport::RequestMsg, tu::DialogId, Handlers};

#[derive(Debug)]
pub struct DialogSm {
    pub id: DialogId,
    pub call_id: rsip::headers::CallId,
    pub transaction_id: String,
    pub local_tag: rsip::common::param::Tag,
    pub local_seqn: u32,
    pub local_uri: rsip::Uri,
    pub remote_tag: rsip::common::param::Tag,
    pub remote_seqn: u32,
    pub remote_uri: rsip::Uri,
    pub remote_target: rsip::Uri,
    pub secure: bool,
    pub route_set: Vec<UriWithParams>,
    pub session_type: SessionType,
    pub contact_header: rsip::headers::Contact,
    pub msg: rsip::Request,
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
    Early(Early),         //just created
    UnAcked(Confirmed),   //received Ack
    Confirmed(Confirmed), //received Ack
    Terminated(Terminated),
    Errored(Errored),
}

//TODO: remove unused async in private functions
#[allow(dead_code)]
impl DialogSm {
    pub async fn new(handlers: Handlers, mut msg: rsip::Response) -> Result<Self, Error> {
        //TODO: need to make sure that it is a success or early
        //TODO: need to make sure that a contact address is added
        //TODO: need to check SIP or SIPS url
        //validations::run(&msg)?;

        let route_set: Vec<UriWithParams> = msg
            .record_route_header()
            .map(|h| h.typed().map(|h| h.uris().to_owned()))
            .transpose()?
            .unwrap_or_else(|| vec![]);
        msg.to_header_mut().mut_tag(Default::default())?;

        let me = Self {
            id: msg.dialog_id()?,
            call_id: msg.call_id_header()?.clone(),
            transaction_id: msg.transaction_id()?.expect("transaction id").into(),
            local_tag: msg
                .to_header()?
                .tag()?,
            local_seqn: msg.cseq_header()?.seq()?,
            local_uri: msg.to_header()?.uri()?,
            remote_tag: msg.from_header()?.tag()?,
            remote_seqn: msg.cseq_header()?.seq()?,
            remote_uri: msg.from_header()?.uri()?,
            remote_target: None,
            route_set,
            session_type: session_type(&msg)?,
            //TODO: need to check transport as well (arrived from TLS?)
            secure: msg.uri.is_sips()?,
            contact_header: msg.contact_header()?.clone(),
            msg: msg.clone(),
            state: DialogState::Unconfirmed(Default::default()),
            created_at: Instant::now(),
            handlers: handlers.clone(),
        };

        handlers
            .transaction
            .new_uac_invite(request_msg_from(msg).await)
            .await?;

        Ok(me)
    }

    async fn _process_incoming_request(&mut self, request: rsip::Request) -> Result<(), Error> {
        if !matches!(self.state, DialogState::Confirmed(_)) {
            return Err(Error::custom(format!(
                "cannot process a request while UAC dialog state is in {}",
                self.state
            )));
        }

        let request = self.set_dialog_defaults_for(request)?;

        match request.method {
            rsip::Method::Invite => {
                self.handlers
                    .transaction
                    .new_uac_invite(request_msg_from(request).await)
                    .await?
            }
            rsip::Method::Bye => {
                self.terminate(request.clone().into());
                self.handlers
                    .transaction
                    .new_uac(request_msg_from(request).await)
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

    async fn _process_incoming_response(&mut self, msg: rsip::Response) -> Result<(), Error> {
        match msg.status_code().kind() {
            rsip::StatusCodeKind::Provisional => self.early(msg).await,
            rsip::StatusCodeKind::Successful => {
                self.confirm(msg.clone()).await?;
                self.handlers
                    .transport
                    .send(
                        request_msg_from(self.msg.ack_request_from(msg))
                            .await
                            .into(),
                    )
                    .await?;
            }
            rsip::StatusCodeKind::Redirection => self.error(
                format!(
                    "({}): received status {}, peer wants redirection to {}",
                    self.id,
                    msg.status_code,
                    msg.contact_header()
                        .map(|h| h.to_string())
                        .unwrap_or_else(|_| "no contact header".into()),
                ),
                Some(msg.into()),
            ),
            _ => self.error(
                format!(
                    "({}): unknown match: {}, {}",
                    self.id, msg.status_code, self.state,
                ),
                Some(msg.into()),
            ),
        };

        Ok(())
    }

    async fn _process_outgoing_request(&mut self, _: rsip::Request) -> Result<(), Error> {
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
            return Ok(self.wrong_transition("confirm", response.into()));
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
        self.local_seqn = self.local_seqn + 1;
        self.local_seqn
    }

    fn set_dialog_defaults_for(
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

    pub async fn process_incoming_request(&mut self, msg: rsip::Request) {
        if let Err(err) = self._process_incoming_request(msg).await {
            self.error(
                format!(
                    "Dialog {} failed to process incoming request: {}",
                    self.id, err
                ),
                None,
            );
        }
    }

    pub async fn process_incoming_response(&mut self, msg: rsip::Response) {
        if let Err(err) = self._process_incoming_response(msg).await {
            self.error(
                format!(
                    "Dialog {} failed to process incoming response: {}",
                    self.id, err
                ),
                None,
            );
        }
    }

    pub async fn process_outgoing_request(&mut self, msg: rsip::Request) {
        if let Err(err) = self._process_outgoing_request(msg).await {
            self.error(
                format!(
                    "Dialog {} failed to process outgoing request: {}",
                    self.id, err
                ),
                None,
            );
        }
    }

    pub async fn process_outgoing_response(&mut self, msg: rsip::Response) {
        if let Err(err) = self._process_outgoing_response(msg).await {
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

async fn request_msg_from(request: rsip::Request) -> RequestMsg {
    let target = resolve_address_for(request.uri.clone()).await;

    RequestMsg {
        sip_request: request,
        peer: target.socket_addr(),
        transport: rsip::Transport::Udp,
    }
}

async fn resolve_address_for(uri: rsip::Uri) -> common::rsip_dns::Target {
    use common::rsip_dns::{self, trust_dns_resolver::TokioAsyncResolver, ResolvableExt};

    let context = rsip_dns::Context::initialize_from(
        uri,
        rsip_dns::AsyncTrustDnsClient::new(
            TokioAsyncResolver::tokio(Default::default(), Default::default()).unwrap(),
        ),
        rsip_dns::SupportedTransports::only(vec![rsip::Transport::Udp]),
    )
    .unwrap();

    let mut lookup = rsip_dns::Lookup::from(context);

    lookup
        .resolve_next()
        .await
        .expect("next Target in dns lookup")
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
