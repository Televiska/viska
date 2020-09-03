use common::futures::SinkExt;
use common::futures_util::stream::StreamExt;
use common::tokio_util::codec::BytesCodec;
use common::tokio_util::udp::UdpFramed;
use processor::Processor;
use tokio::net::UdpSocket;

pub async fn start() -> Result<(), crate::Error> {
    let socket = UdpSocket::bind("0.0.0.0:5060").await?;
    common::log::debug!("starting udp server listening in port 5060");
    let socket = UdpFramed::new(socket, BytesCodec::new());
    let (mut sink, mut stream) = socket.split();

    let processor = Processor::new(); //this should be initialized elsewhere and injected probably

    while let Some(request) = stream.next().await {
        match request {
            Ok((request, addr)) => {
                let response = processor.process_message(request.freeze()).await;
                common::log::info!("{}", addr);
                match response {
                    Ok(response) => sink.send((response, addr)).await?,
                    Err(e) => common::log::error!("{}", e.to_string()),
                };
            }
            Err(e) => common::log::error!("{:?}", e),
        }
    }

    Ok(())
}
