use models::transport::TransportMsg;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct Messages(pub Mutex<Vec<TransportMsg>>);
impl Messages {
    pub async fn len(&self) -> usize {
        self.0.lock().await.len()
    }

    pub async fn first(&self) -> TransportMsg {
        self.0
            .lock()
            .await
            .first()
            .expect("missing first message")
            .clone()
    }

    pub async fn last(&self) -> TransportMsg {
        self.0
            .lock()
            .await
            .last()
            .expect("missing last message")
            .clone()
    }

    pub async fn push(&self, msg: TransportMsg) {
        let mut messages = self.0.lock().await;
        messages.push(msg);
    }
}

impl Default for Messages {
    fn default() -> Self {
        Self(Mutex::new(vec![]))
    }
}
