use common::tokio_util::codec::{BytesCodec, Framed};
use tokio::net::TcpListener;
use tokio::stream::StreamExt;

#[tokio::main]
async fn main() {
    common::pretty_env_logger::init_timed();

    let mut listener = TcpListener::bind("0.0.0.0:5060")
        .await
        .expect("failed to bind socket");

    common::log::debug!("starting server listening in port 5060");
    while let Some(stream) = listener.next().await {
        match stream {
            Ok(stream) => {
                common::log::debug!("new client! {:?}", stream);

                let mut transport = Framed::new(stream, BytesCodec::new());
                while let Some(request) = transport.next().await {
                    match request {
                        Ok(request) => common::log::debug!("{:?}", request),
                        Err(e) => common::log::debug!("{:?}", e),
                    }
                }
            }
            Err(e) => common::log::warn!("{:?}", e),
        }
    }
}
