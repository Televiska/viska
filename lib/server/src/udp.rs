use common::bytes::Bytes;
use common::futures::SinkExt;
use common::futures_util::stream::StreamExt;
use common::libsip;
//use common::log::debug;
use common::nom::error::VerboseError;
use common::tokio_util::codec::BytesCodec;
use common::tokio_util::udp::UdpFramed;
//use helpers::pretty_print;
use tokio::net::UdpSocket;

pub async fn start() {
    let socket = UdpSocket::bind("0.0.0.0:5060")
        .await
        .expect("binding udp socket");
    common::log::debug!("starting udp server listening in port 5060");
    let socket = UdpFramed::new(socket, BytesCodec::new());
    let (mut sink, mut stream) = socket.split();

    while let Some(request) = stream.next().await {
        match request {
            Ok((request, addr)) => {
                let response = process_request(request).await;
                common::log::info!("{}", addr);
                match response {
                    Ok(response) => sink
                        .send((Bytes::from(response), addr))
                        .await
                        .expect("failed"),
                    Err(e) => common::log::error!("{}", e.to_string()),
                };
            }
            Err(e) => common::log::error!("{:?}", e),
        }
    }
}
/*
async fn foo() -> Vec<u8> {
    use tokio::prelude::*; // for read_to_end()

    let mut file = File::open("logs/response.sip")
        .await
        .expect("response file");
    let mut contents = vec![];
    file.read_to_end(&mut contents).await.expect("read file");

    contents
}*/

async fn process_request(request: common::bytes::BytesMut) -> Result<Vec<u8>, String> {
    use std::fs::OpenOptions;
    use std::io::prelude::*;

    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open("my-file")
        .unwrap();
    writeln!(
        file,
        "{}",
        String::from_utf8(request.to_vec()).expect("bytes to string")
    )
    .expect("write to file");
    //debug!("{}", pretty_print(request.to_vec()));

    let (_, request) = libsip::parse_message::<VerboseError<&[u8]>>(&request.to_vec())
        .map_err(|e| e.to_string())?;
    let response = processor::get_response(request).await?;
    writeln!(file, "{}", response.clone()).expect("write to file");
    let response = format!("{}", response).into_bytes();
    //debug!("{}", pretty_print(response.clone()));

    Ok(response)
}
