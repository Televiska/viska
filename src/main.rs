#[tokio::main]
async fn main() {
    common::pretty_env_logger::init_timed();
    common::Config::verify();

    let udp = tokio::spawn(async move {
        server::UdpServer::new()
            .await
            .expect("failed to start udp server")
            .run()
            .await
    });

    tokio::try_join!(udp).expect("try join failed");
}
