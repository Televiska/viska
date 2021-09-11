pub use crate::{presets, Error, ReqProcessor, SipManager};
use common::rsip::{self, headers::typed};
use models::transport::RequestMsg;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::{Arc, Weak},
};

#[derive(Debug)]
pub struct RegisterConfig {
    pub upstream: rsip::HostWithPort,
    pub downstream: rsip::HostWithPort,
    pub auth: rsip::Auth,
    pub scheme: rsip::Scheme,
    pub expiration: Option<u32>,
    pub refresh_interval: Option<u32>
}

#[derive(Debug)]
pub struct Register {
    sip_manager: Weak<SipManager>,
    config: RegisterConfig,
}

impl Register {
    pub fn new(sip_manager: Weak<SipManager>, config: RegisterConfig) -> Self {
        Self {
            sip_manager,
            config,
        }
    }

    fn sip_manager(&self) -> Arc<SipManager> {
        self.sip_manager.upgrade().expect("sip manager is missing!")
    }

    pub async fn send_registration_request(&self) {
        self.sip_manager()
            .transport
            .send(self.delete_all_request().into())
            .await
            .expect("foo");
        self.sip_manager()
            .transport
            .send(self.create_request().into())
            .await
            .expect("foo");
    }

    fn delete_all_request(&self) -> RequestMsg {
        use rsip::headers::untyped;

        let mut headers = self.registration_headers();
        headers.unique_push(untyped::Contact::from("*").into());
        headers.unique_push(untyped::Expires::from(0).into());

        let request = rsip::Request {
            version: rsip::Version::V2,
            method: rsip::Method::Register,
            uri: (rsip::Scheme::Sip, self.config.upstream.host.clone()).into(),
            headers,
            body: vec![],
        };

        //todo use config.upstream.host here in combination with rsip-dns
        RequestMsg {
            sip_request: request,
            peer: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 0, 30)), 5060),
            transport: rsip::Transport::Udp,
        }
    }

    fn create_request(&self) -> RequestMsg {
        let request = rsip::Request {
            version: rsip::Version::V2,
            method: rsip::Method::Register,
            uri: (rsip::Scheme::Sip, self.config.upstream.host.clone()).into(),
            headers: self.registration_headers(),
            body: vec![],
        };

        //todo use config.upstream.host here in combination with rsip-dns
        RequestMsg {
            sip_request: request,
            peer: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 0, 30)), 5060),
            transport: rsip::Transport::Udp,
        }
    }

    fn registration_headers(&self) -> rsip::Headers {
        use common::rsip::headers::untyped;

        let mut headers = rsip::Headers::default();
        headers.push(self.via_header().into());
        headers.push(untyped::MaxForwards::default().into());
        headers.push(self.from_header().into());
        headers.push(self.to_header().into());
        headers.push(untyped::CallId::default().into());
        headers.push(typed::CSeq::from((1, rsip::Method::Register)).into());
        headers.push(self.contact_header().into());
        headers.push(untyped::ContentLength::default().into());

        headers
    }

    fn via_header(&self) -> typed::Via {
        use rsip::param::Branch;

        typed::Via {
            version: rsip::Version::V2,
            transport: rsip::Transport::Udp,
            uri: self.config.downstream.clone().into(),
            params: vec![Branch::default().into()],
        }
    }

    fn from_header(&self) -> typed::From {
        use rsip::param::Tag;

        typed::From {
            display_name: None,
            uri: rsip::Uri {
                scheme: Some(self.config.scheme.clone()),
                auth: Some(self.config.auth.clone()),
                host_with_port: self.config.upstream.host.clone().into(),
                ..Default::default()
            },
            params: vec![Tag::default().into()],
        }
    }

    fn to_header(&self) -> typed::To {
        typed::To {
            display_name: None,
            uri: rsip::Uri {
                scheme: Some(self.config.scheme.clone()),
                auth: Some(self.config.auth.clone()),
                host_with_port: self.config.upstream.host.clone().into(),
                ..Default::default()
            },
            params: vec![],
        }
    }

    fn contact_header(&self) -> typed::Contact {
        typed::Contact {
            display_name: None,
            uri: rsip::Uri {
                scheme: Some(self.config.scheme.clone()),
                auth: Some(self.config.auth.clone()),
                host_with_port: self.config.downstream.clone().into(),
                ..Default::default()
            },
            params: vec![],
        }
    }
}
