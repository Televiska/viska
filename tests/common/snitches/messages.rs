//use common::rsip;
//use models::transport::{RequestMsg, ResponseMsg, TransportMsg};
//use std::convert::TryInto;
use common::rsip;
use models::transport::{RequestMsg, ResponseMsg, TransportLayerMsg};
use std::convert::TryInto;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct Messages<T>(pub Mutex<Vec<T>>);
impl<T: Clone> Messages<T> {
    pub async fn len(&self) -> usize {
        self.0.lock().await.len()
    }

    pub async fn first(&self) -> T {
        self.0
            .lock()
            .await
            .first()
            .expect("missing first message")
            .clone()
    }

    pub async fn last(&self) -> T {
        self.0
            .lock()
            .await
            .last()
            .expect("missing last message")
            .clone()
    }
}

impl<T> Messages<T> {
    pub async fn push(&self, msg: T) {
        let mut messages = self.0.lock().await;
        messages.push(msg);
    }
}

impl<T> Default for Messages<T> {
    fn default() -> Self {
        Self(Mutex::new(vec![]))
    }
}

impl Messages<TransportLayerMsg> {
    pub async fn first_request_msg(&self) -> RequestMsg {
        match self.first().await {
            TransportLayerMsg::Outgoing(msg) => {
                TryInto::<RequestMsg>::try_into(msg).expect("convert to RequestMsg")
            }
            _ => panic!("other"),
        }
    }

    pub async fn first_request(&self) -> rsip::Request {
        match self.first().await {
            TransportLayerMsg::Outgoing(msg) => {
                TryInto::<RequestMsg>::try_into(msg)
                    .expect("convert to RequestMsg")
                    .sip_request
            }
            _ => panic!("other"),
        }
    }

    pub async fn first_response_msg(&self) -> ResponseMsg {
        match self.first().await {
            TransportLayerMsg::Outgoing(msg) => {
                TryInto::<ResponseMsg>::try_into(msg).expect("convert to RequestMsg")
            }
            _ => panic!("other"),
        }
    }

    pub async fn first_response(&self) -> rsip::Response {
        match self.first().await {
            TransportLayerMsg::Outgoing(msg) => {
                TryInto::<ResponseMsg>::try_into(msg)
                    .expect("convert to RequestMsg")
                    .sip_response
            }
            _ => panic!("other"),
        }
    }

    pub async fn last_request_msg(&self) -> RequestMsg {
        match self.last().await {
            TransportLayerMsg::Outgoing(msg) => {
                TryInto::<RequestMsg>::try_into(msg).expect("convert to RequestMsg")
            }
            _ => panic!("other"),
        }
    }

    pub async fn last_request(&self) -> rsip::Request {
        match self.last().await {
            TransportLayerMsg::Outgoing(msg) => {
                TryInto::<RequestMsg>::try_into(msg)
                    .expect("convert to RequestMsg")
                    .sip_request
            }
            _ => panic!("other"),
        }
    }

    pub async fn last_response_msg(&self) -> ResponseMsg {
        match self.last().await {
            TransportLayerMsg::Outgoing(msg) => {
                TryInto::<ResponseMsg>::try_into(msg).expect("convert to RequestMsg")
            }
            _ => panic!("other"),
        }
    }

    pub async fn last_response(&self) -> rsip::Response {
        match self.last().await {
            TransportLayerMsg::Outgoing(msg) => {
                TryInto::<ResponseMsg>::try_into(msg)
                    .expect("convert to RequestMsg")
                    .sip_response
            }
            _ => panic!("other"),
        }
    }
}
