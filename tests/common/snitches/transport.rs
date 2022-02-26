use super::Messages;
use models::{receivers::TrReceiver, transport::TransportLayerMsg, Handlers};
use sip_server::Error;
use std::{time::Duration, sync::Arc};
use crate::common::delay_for;
use common::tokio::sync::mpsc::Receiver;

//TODO: it can be simplified ?
#[derive(Debug)]
pub struct TransportSnitch<T> {
    pub inner: Arc<Inner<T>>,
}

#[derive(Debug)]
pub struct Inner<T> {
    handlers: models::Handlers,
    pub messages: Arc<Messages<T>>,
}

impl<T: Clone + Send + 'static> TransportSnitch<T> {
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

impl<T: Clone> Inner<T> {
    async fn run(&self, mut messages: Receiver<T>) {
        while let Some(msg) = messages.recv().await {
            self.messages.push(msg).await;
        }
    }
}

/*
#[derive(Debug)]
pub struct TransportErrorSnitch {
    sip_manager: Weak<SipManager>,
    fail_switch: Mutex<bool>,
    pub messages: Messages,
}

#[async_trait]
impl TransportLayer for TransportErrorSnitch {
    fn new(sip_manager: Weak<SipManager>) -> Result<Self, Error> {
        Ok(Self {
            sip_manager: sip_manager.clone(),
            fail_switch: Mutex::new(true),
            messages: Default::default(),
        })
    }

    async fn process_incoming_message(&self, _: UdpTuple) -> Result<(), Error> {
        match *self.fail_switch.lock().await {
            true => Err("this is just a snitch".into()),
            false => Ok(()),
        }
    }

    async fn send(&self, msg: TransportMsg) -> Result<(), Error> {
        match *self.fail_switch.lock().await {
            true => Err("this is just a snitch".into()),
            false => {
                self.messages.push(msg).await;
                Ok(())
            }
        }
    }

    async fn run(&self) {}

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl TransportErrorSnitch {
    pub async fn turn_fail_switch_off(&self) {
        let mut switch = self.fail_switch.lock().await;
        *switch = false;
    }

    pub async fn turn_fail_switch_on(&self) {
        let mut switch = self.fail_switch.lock().await;
        *switch = true;
    }
}

#[derive(Debug)]
pub struct TransportPanic;

#[async_trait]
impl TransportLayer for TransportPanic {
    fn new(sip_manager: Weak<SipManager>) -> Result<Self, Error> {
        Ok(Self)
    }

    async fn process_incoming_message(&self, _: UdpTuple) -> Result<(), Error> {
        p!(self)
    }

    async fn send(&self, msg: TransportMsg) -> Result<(), Error> {
        p!(self)
    }

    async fn run(&self) {}

    fn as_any(&self) -> &dyn Any {
        self
    }
}*/
