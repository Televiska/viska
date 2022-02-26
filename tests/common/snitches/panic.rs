use super::Messages;
use crate::common::delay_for;
use common::tokio::sync::mpsc::Receiver;
use models::Handlers;
use sip_server::Error;
use std::{sync::Arc, time::Duration, fmt::Debug};

#[derive(Debug)]
pub struct PanicSnitch<T> {
    pub inner: Arc<Inner<T>>,
}

#[derive(Debug)]
pub struct Inner<T> {
    #[allow(dead_code)]
    handlers: models::Handlers,
    pub messages: Arc<Messages<T>>,
}

impl<T: Clone + Send + Debug + 'static> PanicSnitch<T> {
    pub fn new(handlers: Handlers, messages_rx: Receiver<T>) -> Result<Self, Error> {
        let me = Self {
            inner: Arc::new(Inner {
                handlers,
                messages: Default::default(),
            }),
        };

        me.run(messages_rx);

        Ok(me)
    }

    fn run(&self, messages: Receiver<T>) {
        let inner = self.inner.clone();
        tokio::spawn(async move { inner.run(messages).await });
    }

    pub async fn messages(&self) -> Arc<Messages<T>> {
        delay_for(Duration::from_millis(1)).await;
        self.inner.messages.clone()
    }
}

impl<T: Clone + Debug> Inner<T> {
    async fn run(&self, mut messages: Receiver<T>) {
        while let Some(msg) = messages.recv().await {
            p!("PanicSnitch", msg)
        }
    }
}
