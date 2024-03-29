use super::Messages;
use crate::common::delay_for;
use common::tokio::sync::mpsc::Receiver;
use models::Handlers;
use sip_server::Error;
use std::{sync::Arc, time::Duration};

#[derive(Debug)]
pub struct SpySnitch<T> {
    pub inner: Arc<Inner<T>>,
}

#[derive(Debug)]
pub struct Inner<T> {
    handlers: models::Handlers,
    pub messages: Arc<Messages<T>>,
}

impl<T: Send + 'static> SpySnitch<T> {
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

    pub fn handlers(&self) -> models::Handlers {
        self.inner.handlers.clone()
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

impl<T> Inner<T> {
    async fn run(&self, mut messages: Receiver<T>) {
        while let Some(msg) = messages.recv().await {
            self.messages.push(msg).await;
        }
    }
}
