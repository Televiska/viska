//use common::rsip;
//use models::transport::{RequestMsg, ResponseMsg, TransportMsg};
//use std::convert::TryInto;
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
/*
    pub async fn first_request_msg(&self) -> T {
        TryInto::<RequestMsg>::try_into(self.first().await).expect("convert to RequestMsg")
    }

    pub async fn first_request(&self) -> rsip::Request {
        TryInto::<rsip::Request>::try_into(self.first().await.sip_message)
            .expect("convert to rsip::Request")
    }

    pub async fn first_response_msg(&self) -> ResponseMsg {
        TryInto::<ResponseMsg>::try_into(self.first().await).expect("convert to ResponseMsg")
    }

    pub async fn first_response(&self) -> rsip::Response {
        TryInto::<rsip::Response>::try_into(self.first().await.sip_message)
            .expect("convert to rsip::Response")
    }
*/

    pub async fn last(&self) -> T {
        self.0
            .lock()
            .await
            .last()
            .expect("missing last message")
            .clone()
    }
/*
    pub async fn last_request_msg(&self) -> RequestMsg {
        TryInto::<RequestMsg>::try_into(self.last().await).expect("convert to RequestMsg")
    }

    pub async fn last_request(&self) -> rsip::Request {
        TryInto::<rsip::Request>::try_into(self.last().await.sip_message)
            .expect("convert to rsip::Request")
    }

    pub async fn last_response_msg(&self) -> ResponseMsg {
        TryInto::<ResponseMsg>::try_into(self.last().await).expect("convert to ResponseMsg")
    }

    pub async fn last_response(&self) -> rsip::Response {
        TryInto::<rsip::Response>::try_into(self.last().await.sip_message)
            .expect("convert to rsip::Response")
    }
*/

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
