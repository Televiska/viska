mod processor;

use common::futures_util::stream::StreamExt;
use models::{transport::TransportMsg, ChannelOf};
use tokio::sync::mpsc::{self, Receiver, Sender};

#[allow(dead_code)]
pub struct Core {
    transport_sink: Sender<TransportMsg>,
    core_sink: Sender<TransportMsg>,
    processor: processor::Processor,
}

// listens to core_stream and acts, might send message to transport_sink
impl Core {
    pub async fn spawn(
        transport_sink: Sender<TransportMsg>,
    ) -> Result<Sender<TransportMsg>, crate::Error> {
        let (core_sink, core_stream): ChannelOf<TransportMsg> = mpsc::channel(100);

        let core_sink_cloned = core_sink.clone();
        tokio::spawn(async move {
            let mut core = Self {
                core_sink,
                transport_sink,
                processor: processor::Processor::new(),
            };
            core.run(core_stream).await;
        });

        Ok(core_sink_cloned)
    }

    async fn run(&mut self, mut core_stream: Receiver<TransportMsg>) {
        loop {
            if let Some(transport_tuple) = core_stream.next().await {
                common::log::debug!("Received: {}", transport_tuple.sip_message.debug_compact());
                self.handle_transport_msg(transport_tuple).await;
            }
        }
    }

    async fn handle_transport_msg(&mut self, transport_tuple: TransportMsg) {
        let TransportMsg {
            sip_message,
            peer,
            transport,
        } = transport_tuple;
        match self.processor.process_message(sip_message).await {
            Ok(sip_message) => {
                if self
                    .transport_sink
                    .send(TransportMsg {
                        sip_message,
                        peer,
                        transport,
                    })
                    .await
                    .is_err()
                {
                    common::log::error!("failed to send to transport layer");
                }
            }
            Err(error) => common::log::error!("failed to process msg in core: {}", error),
        }
    }
}
