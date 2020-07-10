use common::libsip;
use common::nom::error::VerboseError;
use common::tokio_util::codec::{BytesCodec, Framed};
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::stream::StreamExt;

pub async fn start() {
    let mut listener = TcpListener::bind("0.0.0.0:5061")
        .await
        .expect("failed to bind socket");

    common::log::debug!("starting server listening in port 5060");
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
                debug_sip_message(request.to_vec());
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

fn debug_sip_message(request: Vec<u8>) {
    let vec: Vec<u8> = request.to_vec();
    common::log::debug!(
        r#"
##################################################
{}
##################################################
"#,
        String::from_utf8(vec).expect("bytes to string")
    );
}
