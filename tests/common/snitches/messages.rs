use crate::common::extensions::TryClone;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct Messages<T>(pub Mutex<Vec<T>>);
impl<T> Messages<T> {
    pub async fn len(&self) -> usize {
        self.0.lock().await.len()
    }
}

impl<T: TryClone> Messages<T> {
    pub async fn try_first(&self) -> T {
        self.0
            .lock()
            .await
            .first()
            .expect("missing first message")
            .try_clone()
            .expect("try_clone")
    }

    pub async fn try_last(&self) -> T {
        self.0
            .lock()
            .await
            .last()
            .expect("missing first message")
            .try_clone()
            .expect("try_clone")
    }

    pub async fn try_latest(&self) -> T {
        self.try_last().await
    }
}

impl<T: Clone> Messages<T> {
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

    pub async fn latest(&self) -> T {
        self.last().await
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
