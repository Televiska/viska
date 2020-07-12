use common::libsip;
use common::nom::error::VerboseError;
use common::tokio_util::codec::{BytesCodec, Framed};
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::stream::StreamExt;

pub async fn start() {
    let mut listener = TcpListener::bind("0.0.0.0:5060")
        .await
        .expect("failed to bind socket");

    common::log::debug!("starting tcp server listening in port 5060");
    while let Some(stream) = listener.next().await {
        match stream {
            Ok(stream) => {
                process_request(stream).await;
            }
            Err(e) => common::log::warn!("{:?}", e),
        }
    }
}

async fn process_request(stream: TcpStream) {
    common::log::debug!("new client! {:?}", stream);

    let mut transport = Framed::new(stream, BytesCodec::new());
    while let Some(request) = transport.next().await {
        match request {
            Ok(request) => {
                helpers::debug_sip_message(request.to_vec(), "tcp".into());
                match libsip::parse_message::<VerboseError<&[u8]>>(&request.to_vec()) {
                    Ok((_, _msg)) => {
                        common::log::debug!("{:?}", "foo");
                    }
                    Err(e) => {
                        common::log::debug!("{:?}", e);
                    }
                };
            }
            Err(e) => common::log::debug!("{:?}", e),
        }
    }
}
