pub mod uac_tests;
pub mod uas_tests;

use models::{server::UdpTuple, transport::TransportMsg, ChannelOf};
use processor::transport::Processor;
use tokio::sync::mpsc::{self, Receiver};

pub struct Setup {
    processor: Processor,
    transport_to_core_stream: Receiver<TransportMsg>,
    transport_to_transaction_stream: Receiver<TransportMsg>,
    transport_to_server_stream: Receiver<UdpTuple>,
}

pub fn setup() -> Setup {
    let (transport_to_core_sink, transport_to_core_stream): ChannelOf<TransportMsg> =
        mpsc::channel(100);

    let (transport_to_transaction_sink, transport_to_transaction_stream): ChannelOf<TransportMsg> =
        mpsc::channel(100);

    let (transport_to_server_sink, transport_to_server_stream): ChannelOf<UdpTuple> =
        mpsc::channel(100);

    let processor = Processor::new(
        transport_to_core_sink,
        transport_to_transaction_sink,
        transport_to_server_sink,
    );

    Setup {
        processor,
        transport_to_core_stream,
        transport_to_transaction_stream,
        transport_to_server_stream,
    }
}
